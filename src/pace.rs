//! Adaptive request pacing: AIMD on request *rate* (req/s), as in TCP congestion
//! control. Rate limits are about requests per time window, not about how many
//! are in flight, so we pace request starts instead of capping concurrency.
//!
//! - Success: additive increase. Fast while far below the last known ceiling,
//!   very slow near it, so the rate hovers just under the limit.
//! - Throttle: halve the rate, remember it as the ceiling, and pause everyone
//!   (exponential backoff with jitter, or the server's Retry-After). Only one
//!   cut per loss event: requests started before the last cut are ignored, so a
//!   burst of simultaneous failures doesn't collapse the rate to the floor.
//! - Surviving above the ceiling raises it, so the limit is re-probed.
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

// DDG's block is a sticky cooldown (bare 403, no Retry-After) lasting minutes,
// so back off from 5s up to 5min rather than the usual 1s..1min.
const BASE_BACKOFF_SECS: u64 = 5;
const MAX_BACKOFF: Duration = Duration::from_secs(300);

pub struct Pacer {
    rate: f64,
    ceiling: f64,
    min_rate: f64,
    max_rate: f64,
    next_start: Instant,
    pause_until: Instant,
    last_cut: Option<Instant>,
    strikes: u32,
}

impl Pacer {
    pub fn new(initial_rate: f64, max_rate: f64, now: Instant) -> Self {
        let min_rate = 0.25;
        Pacer {
            rate: initial_rate.clamp(min_rate, max_rate),
            ceiling: f64::INFINITY,
            min_rate,
            max_rate,
            next_start: now,
            pause_until: now,
            last_cut: None,
            strikes: 0,
        }
    }

    pub fn rate(&self) -> f64 {
        self.rate
    }

    /// Claim the next start slot; the caller sleeps until the returned instant.
    pub fn reserve(&mut self, now: Instant) -> Instant {
        let start = now.max(self.next_start).max(self.pause_until);
        self.next_start = start + Duration::from_secs_f64(1.0 / self.rate);
        start
    }

    pub fn on_success(&mut self) {
        self.strikes = 0;
        if self.rate > self.ceiling {
            self.ceiling = self.rate; // survived above it: the limit was higher
        }
        let step = if self.rate >= 0.8 * self.ceiling { 0.02 } else { 0.5 };
        // +step req/s per second of traffic (each success is 1/rate seconds).
        self.rate = (self.rate + step / self.rate).min(self.max_rate);
    }

    /// `started` is when the throttled request was sent.
    pub fn on_throttle(&mut self, started: Instant, now: Instant, retry_after: Option<Duration>) {
        if matches!(self.last_cut, Some(cut) if started <= cut) {
            return; // same loss event as a cut we already made
        }
        self.last_cut = Some(now);
        self.ceiling = self.rate;
        self.rate = (self.rate * 0.5).max(self.min_rate);
        self.strikes += 1;
        let base = retry_after.unwrap_or_else(|| Duration::from_secs(BASE_BACKOFF_SECS << (self.strikes - 1).min(6)));
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.subsec_nanos()).unwrap_or(0);
        let jitter = 0.75 + 0.5 * (nanos % 1000) as f64 / 1000.0;
        let delay = base.mul_f64(jitter).min(MAX_BACKOFF);
        self.pause_until = self.pause_until.max(now + delay);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;

    #[test]
    fn reserve_spaces_requests_by_rate() {
        let t0 = Instant::now();
        let mut p = Pacer::new(4.0, 100.0, t0);
        let a = p.reserve(t0);
        let b = p.reserve(t0);
        assert_eq!(b - a, Duration::from_millis(250));
    }

    #[test]
    fn simultaneous_failures_cut_once() {
        let t0 = Instant::now();
        let mut p = Pacer::new(8.0, 100.0, t0);
        let sent = t0;
        let now = t0 + Duration::from_millis(500);
        for _ in 0..10 {
            p.on_throttle(sent, now, None);
        }
        assert_eq!(p.rate(), 4.0);
    }

    #[test]
    fn retry_after_is_honoured() {
        let t0 = Instant::now();
        let mut p = Pacer::new(2.0, 100.0, t0);
        p.on_throttle(t0, t0, Some(Duration::from_secs(30)));
        assert!(p.reserve(t0) - t0 >= Duration::from_secs(20));
    }

    /// Server that throttles when more than `limit` requests land in any 1s window.
    /// The pacer should settle just under the limit with few throttles.
    #[test]
    fn converges_just_under_a_hidden_limit() {
        let t0 = Instant::now();
        let limit = 10usize;
        let mut p = Pacer::new(1.0, 100.0, t0);
        let mut window: VecDeque<Instant> = VecDeque::new();
        let (mut ok, mut throttled) = (0u32, 0u32);
        let mut clock = t0;
        let end = t0 + Duration::from_secs(600);
        while clock < end {
            let start = p.reserve(clock);
            clock = start;
            while matches!(window.front(), Some(&w) if start - w > Duration::from_secs(1)) {
                window.pop_front();
            }
            if window.len() >= limit {
                throttled += 1;
                p.on_throttle(start, start, None);
            } else {
                window.push_back(start);
                ok += 1;
                p.on_success();
            }
        }
        let throughput = ok as f64 / 600.0;
        let frac = throttled as f64 / (ok + throttled) as f64;
        println!("throughput {throughput:.2}/s, throttled {:.1}%, final rate {:.2}", frac * 100., p.rate());
        assert!(frac < 0.05, "throttled {frac}");
        assert!(throughput > 6.0 && throughput <= 10.0, "throughput {throughput}");
    }
}
