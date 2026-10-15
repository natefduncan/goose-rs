use super::grid;
use super::pace::Pacer;
use futures::stream::{FuturesUnordered, StreamExt};
use geo::Coordinate;
use geo_types::Point;
use indicatif::ProgressBar;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashSet, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Coord {
    latitude: Option<f64>,
    longitude: Option<f64>,
}

#[derive(Serialize, Deserialize)]
struct Response {
    results: Vec<Place>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Place {
    address: Option<String>,
    city: Option<String>,
    coordinates: Coord,
    display_phone: Option<String>,
    engine: Option<String>,
    id: Option<String>,
    name: Option<String>,
    phone: Option<String>,
    // Kept as raw JSON: DDG's types for these vary by provider, and a type
    // mismatch would otherwise drop the whole page of results.
    #[serde(default)]
    ddg_category: Option<Value>,
    #[serde(default)]
    website: Option<Value>,
    #[serde(default)]
    rating: Option<Value>,
    #[serde(default)]
    reviews: Option<Value>,
    #[serde(default)]
    price: Option<Value>,
    #[serde(default)]
    hours: Option<Value>,
    #[serde(default)]
    timezone: Option<Value>,
    #[serde(default)]
    closed: Option<Value>,
    #[serde(default)]
    country_code: Option<Value>,
    #[serde(default)]
    facebook_id: Option<Value>,
    #[serde(default)]
    instagram_id: Option<Value>,
    #[serde(default)]
    twitter_id: Option<Value>,
    #[serde(default)]
    yelp_business_id: Option<Value>,
    pub search: Option<String>
}

fn cell(v: &Option<Value>) -> String {
    match v {
        None | Some(Value::Null) => String::new(),
        Some(Value::String(s)) => s.clone(),
        Some(other) => other.to_string(),
    }
}

impl Place {
    /// One CSV row, in the order of `CSV_HEADER`. Nested values (hours) are JSON-encoded.
    pub fn csv_row(&self) -> Vec<String> {
        let opt = |s: &Option<String>| s.clone().unwrap_or_default();
        vec![
            opt(&self.address),
            opt(&self.city),
            self.coordinates.latitude.map(|v| v.to_string()).unwrap_or_default(),
            self.coordinates.longitude.map(|v| v.to_string()).unwrap_or_default(),
            opt(&self.display_phone),
            opt(&self.engine),
            opt(&self.id),
            opt(&self.name),
            opt(&self.phone),
            opt(&self.search),
            cell(&self.ddg_category),
            cell(&self.website),
            cell(&self.rating),
            cell(&self.reviews),
            cell(&self.price),
            cell(&self.hours),
            cell(&self.timezone),
            cell(&self.closed),
            cell(&self.country_code),
            cell(&self.facebook_id),
            cell(&self.instagram_id),
            cell(&self.twitter_id),
            cell(&self.yelp_business_id),
        ]
    }
}

pub const CSV_HEADER: [&str; 23] = [
    "address", "city", "latitude", "longitude", "display_phone", "engine", "id", "name",
    "phone", "search", "ddg_category", "website", "rating", "reviews", "price", "hours",
    "timezone", "closed", "country_code", "facebook_id", "instagram_id", "twitter_id",
    "yelp_business_id",
];

fn get_url(q: &str, g: [Coordinate<f64>; 2]) -> String {
    let encoded_q = utf8_percent_encode(q, NON_ALPHANUMERIC).to_string();
    format!("https://duckduckgo.com/local.js?q={}&tg=maps_places&rt=D&mkexp=b&is_requery=1&bbox_tl={},{}&bbox_br={},{}&strict_bbox=1&wiamr=a&nyexp=b", encoded_q, g[0].y, g[0].x, g[1].y, g[1].x)
}

type Cell = [Coordinate<f64>; 2];

/// How many times a throttled cell is retried before it is abandoned.
const MAX_ATTEMPTS: u32 = 10;
const MILES_PER_DEGREE_LAT: f64 = 69.0;

fn split(c: Cell) -> [Cell; 4] {
    let (mx, my) = ((c[0].x + c[1].x) / 2., (c[0].y + c[1].y) / 2.);
    let p = |x, y| Coordinate { x, y };
    [
        [p(c[0].x, c[0].y), p(mx, my)],
        [p(mx, c[0].y), p(c[1].x, my)],
        [p(c[0].x, my), p(mx, c[1].y)],
        [p(mx, my), p(c[1].x, c[1].y)],
    ]
}

fn cell_miles(c: Cell) -> f64 {
    (c[0].y - c[1].y).abs() * MILES_PER_DEGREE_LAT
}

enum Fetched {
    Places(Vec<Place>),
    /// Rate limited, blocked, or timed out: slow down and retry.
    Throttled(Option<Duration>),
    /// Not worth retrying.
    Failed(String),
}

async fn fetch_cell(client: &Client, q: &str, cell: Cell) -> Fetched {
    let resp = match client.get(get_url(q, cell)).send().await {
        Ok(r) => r,
        Err(e) if e.is_timeout() || e.is_connect() => return Fetched::Throttled(None),
        Err(e) => return Fetched::Failed(e.to_string()),
    };
    let status = resp.status();
    if status.as_u16() == 429 || status.as_u16() == 403 || status.is_server_error() {
        let retry_after = resp
            .headers()
            .get(reqwest::header::RETRY_AFTER)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok())
            .map(Duration::from_secs);
        return Fetched::Throttled(retry_after);
    }
    if !status.is_success() {
        return Fetched::Failed(format!("HTTP {}", status));
    }
    // A 200 that isn't the expected JSON is almost always a block/captcha page.
    match resp.json::<Response>().await {
        Ok(r) => Fetched::Places(r.results),
        Err(_) => Fetched::Throttled(None),
    }
}

fn place_key(p: &Place) -> String {
    match &p.id {
        Some(id) => id.clone(),
        None => format!(
            "{}|{}|{}",
            p.name.clone().unwrap_or_default(),
            p.address.clone().unwrap_or_default(),
            p.city.clone().unwrap_or_default()
        ),
    }
}

/// The four children of one cell, collected until all have reported.
struct Group {
    parent: HashSet<String>,
    kids: Vec<(Cell, HashSet<String>)>,
    remaining: usize,
}

/// DDG's page size is not a reliable truncation signal (pages of 13, 15, 16
/// and 19 all occur in truncated cells), so instead of guessing a threshold we
/// ask whether splitting a cell revealed anything its parent page didn't have.
/// If the children together found places the parent missed, the parent was
/// truncated and its children may be too; if not, the parent was complete.
fn splitting_revealed_more(parent: &HashSet<String>, kids: &[(Cell, HashSet<String>)]) -> bool {
    kids.iter().any(|(_, ids)| ids.iter().any(|id| !parent.contains(id)))
}

/// Fetch every place for `q` in the square of side `distance_miles` around
/// `start_point`. Each starting cell is split once; its children are split
/// again only while splitting keeps revealing new places (see
/// `splitting_revealed_more`), down to `min_cell_miles`. Request starts are
/// paced by `pacer`, which is shared across queries and adapts to the server's
/// rate limit; `max_in_flight` only bounds memory/sockets.
pub async fn query(
    q: &str,
    start_point: &Point<f64>,
    distance_miles: f64,
    max_in_flight: usize,
    min_cell_miles: f64,
    pacer: &Arc<Mutex<Pacer>>,
) -> Vec<Place> {
    let cell_size = (distance_miles / 2.0).clamp(1.0, 20.0);
    // (cell, attempt, group the cell belongs to; None for a starting cell)
    let mut queue: VecDeque<(Cell, u32, Option<usize>)> =
        grid::get_grids(start_point, distance_miles, cell_size)
            .into_iter()
            .map(|c| (c, 0, None))
            .collect();
    let mut groups: Vec<Group> = Vec::new();
    let bar = ProgressBar::new_spinner();
    let client = Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .expect("Failed to build HTTP client");

    let mut output: Vec<Place> = Vec::new();
    let mut in_flight = FuturesUnordered::new();
    let (mut requests, mut throttles, mut failures) = (0u64, 0u64, 0u64);

    // Queue the four children of `cell` as a new group.
    fn descend(
        cell: Cell,
        ids: HashSet<String>,
        groups: &mut Vec<Group>,
        queue: &mut VecDeque<(Cell, u32, Option<usize>)>,
    ) {
        groups.push(Group { parent: ids, kids: Vec::new(), remaining: 4 });
        let g = groups.len() - 1;
        queue.extend(split(cell).iter().map(|&c| (c, 0, Some(g))));
    }

    loop {
        while in_flight.len() < max_in_flight.max(1) {
            let Some((cell, attempt, group)) = queue.pop_front() else { break };
            let (client, pacer) = (&client, pacer);
            in_flight.push(async move {
                let slot = pacer.lock().unwrap().reserve(Instant::now());
                tokio::time::sleep_until(slot.into()).await;
                let started = Instant::now();
                (cell, attempt, group, started, fetch_cell(client, q, cell).await)
            });
        }
        let Some((cell, attempt, group, started, result)) = in_flight.next().await else { break };
        requests += 1;
        // Some(ids) when the cell is finished (fetched or abandoned).
        let mut finished: Option<HashSet<String>> = None;
        match result {
            Fetched::Places(places) => {
                pacer.lock().unwrap().on_success();
                finished = Some(places.iter().map(place_key).collect());
                output.extend(places);
            }
            Fetched::Throttled(retry_after) => {
                throttles += 1;
                pacer.lock().unwrap().on_throttle(started, Instant::now(), retry_after);
                if attempt + 1 < MAX_ATTEMPTS {
                    queue.push_back((cell, attempt + 1, group));
                } else {
                    failures += 1;
                    eprintln!("Giving up on a cell after {} throttled attempts", MAX_ATTEMPTS);
                    finished = Some(HashSet::new());
                }
            }
            Fetched::Failed(e) => {
                failures += 1;
                eprintln!("Failure: {}", e);
                finished = Some(HashSet::new());
            }
        }
        if let Some(ids) = finished {
            let can_split = cell_miles(cell) / 2. >= min_cell_miles;
            match group {
                // Starting cells are always split once if they found anything.
                None => {
                    if !ids.is_empty() && can_split {
                        descend(cell, ids, &mut groups, &mut queue);
                    }
                }
                Some(g) => {
                    groups[g].kids.push((cell, ids));
                    groups[g].remaining -= 1;
                    if groups[g].remaining == 0 {
                        let kids = std::mem::take(&mut groups[g].kids);
                        if can_split && splitting_revealed_more(&groups[g].parent, &kids) {
                            for (c, ids) in kids {
                                if !ids.is_empty() {
                                    descend(c, ids, &mut groups, &mut queue);
                                }
                            }
                        }
                    }
                }
            }
        }
        bar.set_message(format!(
            "{}: {} requests, {} places, {} queued, {:.1} req/s, {} throttled, {} failed",
            q, requests, output.len(), queue.len() + in_flight.len(),
            pacer.lock().unwrap().rate(), throttles, failures
        ));
        bar.tick();
    }
    bar.finish();
    eprintln!("{}: {} requests, {} throttled, {} failed", q, requests, throttles, failures);
    if failures > 0 {
        eprintln!("WARNING: {} cell(s) for '{}' could not be fetched; results are incomplete", failures, q);
    }

    let mut seen: HashSet<String> = HashSet::new();
    output.retain(|place| seen.insert(place_key(place)));
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_quarters_the_cell_exactly() {
        let c: Cell = [Coordinate { x: 0., y: 4. }, Coordinate { x: 4., y: 0. }];
        let q = split(c);
        assert_eq!(q[0], [Coordinate { x: 0., y: 4. }, Coordinate { x: 2., y: 2. }]);
        assert_eq!(q[3], [Coordinate { x: 2., y: 2. }, Coordinate { x: 4., y: 0. }]);
        let area: f64 = q.iter().map(|c| (c[1].x - c[0].x) * (c[0].y - c[1].y)).sum();
        assert_eq!(area, 16.);
    }

    fn ids(v: &[&str]) -> HashSet<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn keeps_splitting_while_children_reveal_new_places() {
        let c: Cell = [Coordinate { x: 0., y: 1. }, Coordinate { x: 1., y: 0. }];
        let parent = ids(&["a", "b"]);
        let kids = vec![(c, ids(&["a", "x"])), (c, ids(&["b"]))];
        assert!(splitting_revealed_more(&parent, &kids));
    }

    #[test]
    fn stops_when_children_only_repartition_the_parent() {
        let c: Cell = [Coordinate { x: 0., y: 1. }, Coordinate { x: 1., y: 0. }];
        let parent = ids(&["a", "b", "c"]);
        let kids = vec![(c, ids(&["a"])), (c, ids(&["b", "c"])), (c, ids(&[])), (c, ids(&[]))];
        assert!(!splitting_revealed_more(&parent, &kids));
    }
}
