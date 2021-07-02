use super::grid;
use futures::{stream, StreamExt};
use geo::Coordinate;
use geo_types::Point;
use indicatif::ProgressBar;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Coord {
    latitude: f64,
    longitude: f64,
}

#[derive(Serialize, Deserialize)]
struct Response {
    results: Vec<Place>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Place {
    address: Option<String>,
    address_lines: Vec<String>,
    city: Option<String>,
    coordinates: Coord,
    display_phone: Option<String>,
    engine: Option<String>,
    id: Option<String>,
    name: Option<String>,
    phone: Option<String>,
}

fn get_url(q: &str, g: [Coordinate<f64>; 2]) -> String {
    let url : String = format!("https://duckduckgo.com/local.js?q={}&tg=maps_places&rt=D&mkexp=b&is_requery=1&bbox_tl={},{}&bbox_br={},{}&strict_bbox=1&wiamr=a&nyexp=b", q, g[0].y, g[0].x, g[1].y, g[1].x);
    return url;
}

#[tokio::main]
pub async fn query(
    q: &str,
    start_point: &Point<f64>,
    distance_miles: f64,
    concurrent_requests: usize,
) {
    let grids = grid::get_grids(&start_point, distance_miles, 5.);
    let mut urls = Vec::new();

    for g in grids {
        urls.push(get_url(q, g));
    }
    let bar_length: u64 = urls.len() as u64;
    let bar = ProgressBar::new(bar_length);
    let client = Client::new();
    let mut bodies = stream::iter(urls)
        .map(|url| {
            let client = &client;
            async move {
                let resp = client.get(url).send().await?;
                resp.json::<Response>().await
            }
        })
        .buffer_unordered(concurrent_requests);

    let mut outfile = tokio::fs::File::create("output.json")
        .await
        .expect("Failed to create file.");
    outfile
        .write("[".as_bytes())
        .await
        .expect("Could not write to file.");
    let mut is_first: u32 = 1;
    while let Some(v) = bodies.next().await {
        let data = match v {
            Ok(data) => data,
            Err(e) => {
                eprintln!("Failure: {}", e);
                Response {
                    results: [].to_vec(),
                }
            }
        };
        bar.inc(1);
        for place in data.results {
            let string = serde_json::to_string(&place).unwrap();
            if is_first == 1 {
                is_first = 0;
            } else {
                outfile
                    .write(",".as_bytes())
                    .await
                    .expect("Could not write to file.");
            }
            outfile
                .write(string.as_bytes())
                .await
                .expect("Could not write to file.");
        }
    }
    outfile
        .write("]".as_bytes())
        .await
        .expect("Could not write to file.");
    bar.finish();
}
