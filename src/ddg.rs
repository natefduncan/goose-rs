use super::grid;
use futures::{stream, StreamExt};
use geo::Coordinate;
use geo_types::Point;
use indicatif::ProgressBar;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::str;
use std::time::Duration;

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
    pub search: Option<String>
}

fn get_url(q: &str, g: [Coordinate<f64>; 2]) -> String {
    let encoded_q = utf8_percent_encode(q, NON_ALPHANUMERIC).to_string();
    format!("https://duckduckgo.com/local.js?q={}&tg=maps_places&rt=D&mkexp=b&is_requery=1&bbox_tl={},{}&bbox_br={},{}&strict_bbox=1&wiamr=a&nyexp=b", encoded_q, g[0].y, g[0].x, g[1].y, g[1].x)
}

pub async fn query(
    q: &str,
    start_point: &Point<f64>,
    distance_miles: f64,
    concurrent_requests: usize,
) -> Vec<Place> {
    let cell_size = (distance_miles / 2.0).clamp(1.0, 20.0);
    let grids = grid::get_grids(&start_point, distance_miles, cell_size);
    let mut urls = Vec::new();
    //Get URLs
    for g in grids {
        urls.push(get_url(q, g));
    }
    let bar_length: u64 = urls.len() as u64;
    let bar = ProgressBar::new(bar_length);
    let client = Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .expect("Failed to build HTTP client");
    //Response stream
    let mut bodies = stream::iter(urls)
        .map(|url| {
            let client = &client;
            async move {
                let resp = client.get(url).send().await?;
                resp.json::<Response>().await
            }
        })
        .buffer_unordered(concurrent_requests);
    let mut output = Vec::new();
    //Add response to outfile.
    while let Some(v) = bodies.next().await {
        let mut data = match v {
            Ok(data) => data,
            Err(e) => {
                eprintln!("Failure: {}", e);
                Response {
                    results: [].to_vec(),
                }
            }
        };
        output.append(&mut data.results);
        bar.inc(1); // Update progress bar
    }
    bar.finish();

    let mut seen_ids: HashSet<String> = HashSet::new();
    let mut seen_fallback: HashSet<(String, String, String)> = HashSet::new();
    output.retain(|place| {
        if let Some(id) = &place.id {
            seen_ids.insert(id.clone())
        } else {
            let key = (
                place.name.clone().unwrap_or_default(),
                place.address.clone().unwrap_or_default(),
                place.city.clone().unwrap_or_default(),
            );
            seen_fallback.insert(key)
        }
    });

    output
}
