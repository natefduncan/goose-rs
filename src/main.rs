extern crate clap;
use clap::{Arg, App};

mod grid;
mod ddg; 
mod geocode; 

fn main() {
    let matches = App::new("Goose")
        .version("1.0")
        .author("Nate D.")
        .about("Query Duck Duck Go to get location data.")
        .arg(Arg::with_name("QUERY")
            .help("Sets your search value (e.g. Restaurant, Park, etc).")
            .required(true)
            .index(1))
        .arg(Arg::with_name("LOCATION")
            .help("Sets your location (e.g. Dallas, TX).")
            .required(true)
            .index(2))
        .arg(Arg::with_name("DISTANCE")
            .help("Sets how far you want to seach in miles.")
            .required(true)
            .index(3))
        .get_matches();

    let query = matches.value_of("QUERY").expect("Unable to parse query");
    let location = matches.value_of("LOCATION").expect("Unable to parse location");
    let distance = matches.value_of("DISTANCE").expect("Unable to parse distance"); 
    let start_point = geocode::geocode(&location); 
    println!("Found coordinates for {}: {}, {}", location, start_point.lat(), start_point.lng()); 
    println!("Searching for {} within {} miles.", query, distance); 
    ddg::query(&query, &start_point, distance.parse::<f64>().unwrap()); 
}
