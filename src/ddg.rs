use super::grid; 
use serde::{Serialize, Deserialize}; 
use geo::{Coordinate}; 
use geo_types::{Point}; 
use futures::{stream, StreamExt};
use reqwest::Client;
use tokio;

#[derive(Serialize, Deserialize, Debug)]
struct Coord {
    latitude : f64, 
    longitude : f64
}

#[derive(Serialize, Deserialize)]
struct Response {
    results : Vec<Place>
}

#[derive(Serialize, Deserialize, Debug)]
struct Place {
    address : Option<String>, 
    address_lines : Vec<String>, 
    city : Option<String>, 
    coordinates : Coord, 
    display_phone : Option<String>, 
    engine: Option<String>,  
    id : Option<String>, 
    name : Option<String>, 
    phone : Option<String>, 
}


pub fn get_url(q : &str, g: [Coordinate<f64>; 2]) -> String {
    let url : String = format!("https://duckduckgo.com/local.js?q={}&tg=maps_places&rt=D&mkexp=b&is_requery=1&bbox_tl={},{}&bbox_br={},{}&strict_bbox=1&wiamr=a&nyexp=b", q, g[0].y, g[0].x, g[1].y, g[1].x); 
    return url; 
}

const CONCURRENT_REQUESTS: usize = 2;

#[tokio::main]
pub async fn query(q : &str, start_point : &Point<f64>, distance_miles : f64) {
    let grids = grid::get_grids(&start_point, distance_miles, 5.);
    let mut urls  = Vec::new(); 
    for g in grids {
        urls.push(get_url(q, g)); 
    }
    let client = Client::new();
    let bodies = stream::iter(urls)
        .map(|url| {
            let client = &client;
            async move {
                let resp = client.get(url).send().await?;
                resp.json::<Response>().await
            }
        })
        .buffer_unordered(CONCURRENT_REQUESTS);

    bodies
        .for_each(|b| async {
            match b {
                Ok(b) => println!("{:?}", b.results),
                Err(e) => eprintln!("Got an error: {}", e),
            }
        })
        .await;
}