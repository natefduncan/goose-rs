# Iteration 1 — 2026-06-08

## Objective
Fix GH issues 1-13 in goose-rs, one per iteration.

## Current state
All 13 issues open. Starting with issue #1: Multiple tokio runtimes created when querying multiple terms.

## Issue #1 Analysis
- `ddg::query` has `#[tokio::main]` which creates a new runtime per call
- `main.rs` calls it in a loop, one per comma-separated query term
- Fix: remove `#[tokio::main]` from `ddg::query`, add it to `main()`, use `join_all` for concurrency

## Plan
1. Remove `#[tokio::main]` from `ddg.rs`
2. Make `main()` async with `#[tokio::main]`
3. Use `futures::future::join_all` for concurrent query execution
4. Commit with "Closes #1" in message
5. Push to main

## Iteration 1 result
Issue #1 fixed and pushed. Removed `#[tokio::main]` from `ddg::query`, moved to `main()`, and used `join_all` for concurrent query execution.

## Next: Issue #2
Title: "Panics on invalid user input instead of graceful errors"
Task created: task-1780934679-b359

## Iteration 2 result — 2026-06-08
Issue #2 fixed and pushed. Added `anyhow = "1"` dependency. Refactored to `run()` async fn returning `Result<()>`, `main()` catches errors and prints clean message. Fixed geocode.rs to return Result and check empty results, files.rs uses `?` instead of unwrap/expect, main.rs validates --distance and --concurrency with descriptive errors.

## Next: Issue #3
Title: "Search query is not URL-encoded before being inserted into request URL"
Task created: task-1780934796-c1d0
Fix: encode query with `percent_encoding::utf8_percent_encode` (transitive dep) or add `urlencoding` crate.

## Iteration 3 result — 2026-06-08
Issue #3 fixed and pushed. Added `percent-encoding = "2"` to Cargo.toml. In `get_url()`, used `utf8_percent_encode(q, NON_ALPHANUMERIC)` to encode the query before inserting into URL.

## Next: Issue #4
Title: "miles_to_degrees constant produces inaccurate bounding boxes at non-equatorial latitudes"
Task created: task-1780934906-6c42
Fix: replace hardcoded `miles_to_degrees` constant with `GeodesicDestination` trait from the `geo` crate (already a dep). Use bearing (0/90/180/270) + miles converted to meters.

## Iteration 4 result — 2026-06-08
Issue #4 fixed and pushed. geo 0.18.0 has no GeodesicDestination, but HaversineDestination is available and equally latitude-accurate. Replaced hardcoded miles_to_degrees with haversine_destination(bearing, meters) where meters = miles * 1609.344 and bearings are 0/90/180/270 for Up/Right/Down/Left. Also note: Offset::Left had a bug using lat_translate instead of lng_translate — this is fixed implicitly by the rewrite.

## Next: Issue #5
Title: "Duplicate places in output due to overlapping grid cell boundaries"
Task created: task-1780935001-d136
Fix: After collecting all results in ddg.rs, deduplicate by Place.id using a HashMap. Fallback for places with no id: deduplicate by (name, address, city).

## Iteration 5 result — 2026-06-08
Issue #5 fixed and pushed. After collecting all grid responses in `query()`, deduplicate using `output.retain()` with a `HashSet<String>` for Place.id and a `HashSet<(String, String, String)>` fallback for (name, address, city) when id is absent.

## Next: Issue #6
Title: "Unknown --file-type value silently produces no output"
Task created: task-1780935076-d0d7
Fix: Add `possible_values(&["csv", "json"])` to the FILE-TYPE clap Arg so invalid values are rejected before any network requests are made.

## Iteration 6 result — 2026-06-08
Issue #6 fixed and pushed. Added `.possible_values(&["csv", "json"])` to the FILE-TYPE clap Arg in main.rs. Invalid values now produce a clap error immediately with "possible values: csv, json" listed, before any geocoding or network requests.

## Next: Issue #7
Title: "No HTTP request timeout — hangs indefinitely on slow/unresponsive DDG API"
Task created: task-1780935131-219f
Fix: Use `Client::builder().timeout(Duration::from_secs(15)).build()` instead of `Client::new()` in ddg.rs.

## Iteration 7 result — 2026-06-08
Issue #7 fixed and pushed. Added `use std::time::Duration` import. Replaced `Client::new()` with `Client::builder().timeout(Duration::from_secs(15)).build().expect(...)` in ddg.rs. Pushed to main.

Issue #8 (Offset::Left lat/lng bug) was already fixed implicitly by the haversine rewrite in iteration 4. Closed it on GitHub with a reference to commit fa6ef1d.

## Next: Issue #9
Title: "Grid cell size is hardcoded to 5 miles regardless of search distance"
Task created: task-1780935217-f61e
Fix: Replace hardcoded `5.` with `(distance_miles / 2.0).clamp(1.0, 20.0)` in ddg.rs get_grids call.

## Iteration 9 result — 2026-06-08
Issue #9 fixed and pushed. In ddg.rs `query()`, replaced hardcoded `5.` with `let cell_size = (distance_miles / 2.0).clamp(1.0, 20.0)` passed to `get_grids`. Short-range searches now use smaller cells; long-range searches scale up to 20-mile max.

## Next: Issue #10
Title: "JSON output builds entire result set in memory as a String before writing"
Task created: task-1780935282-7c42
Fix: Replace manual JSON string assembly in `files::output_as_json` with `serde_json::to_writer(io::stdout(), &data)?`. Also fix `.unwrap()` and remove `is_first` tracking logic.

## Iteration 10 result — 2026-06-08
Issue #10 fixed and pushed. Replaced manual JSON string assembly (push_str loop + is_first tracking) in `files::output_as_json` with `serde_json::to_writer(io::stdout(), &data)?`. Removed unused `Write` import. Commit: 4cd7cea.

## Next: Issue #11
Title: "Using clap v2 (EOL) — upgrade to clap v4"
Task created: task-1780935375-c5e3
Fix: Change `clap = "2.33.3"` to `clap = { version = "4", features = ["derive"] }` in Cargo.toml. Rewrite `main.rs` CLI with `#[derive(Parser)]` struct. Eliminates manual `.parse::<f64>()` calls, uses `value_parser = ["csv", "json"]` for file-type validation.

## Iteration 11 result — 2026-06-08
Issue #11 fixed and pushed. Changed `clap = "2.33.3"` to `clap = { version = "4", features = ["derive"] }`. Rewrote main.rs using `#[derive(Parser)]` struct — eliminates manual parse calls for --distance (f64) and --concurrency (usize), uses `default_value_t`, and `value_parser = ["csv", "json"]` for file-type validation. Commit: 0fc9745.

## Next: Issue #12
Title: "geocoding dependency uses wildcard version (*)"
Task created: task-1780935479-4570
Fix: Change `version = "*"` to `version = "0.4"` (currently resolved version is 0.4.0).

## Iteration 12 result — 2026-06-08
Issue #12 fixed and pushed. Changed `geocoding = { version = "*", ... }` to `version = "0.4"` in Cargo.toml. Build passes. Commit: 4106a73.

## Next: Issue #13
Title: "Unused import: use std::str in ddg.rs"
Task created: task-1780935527-471f
Fix: Remove `use std::str;` line from ddg.rs.

## Iteration 13 result — 2026-06-08
Issue #13 fixed and pushed. Removed `use std::str;` from ddg.rs line 10. Build passes cleanly. Commit: 6c2dfaa.

## OBJECTIVE COMPLETE
All 13 issues (GH_RANGE=1-13) have been fixed and pushed to main. Loop finished.
