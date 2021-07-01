extern crate clap;
use clap::{App, Arg};
use std::time::{Instant};

mod ddg;
mod geocode;
mod grid;

fn main() {
    let matches = App::new("Goose")
        .version("1.0")
        .author("Nate D.")
        .about("Query Duck Duck Go to get location data.")
        .arg(
            Arg::with_name("QUERY")
                .help("Sets your search value (e.g. Restaurant, Park, etc).")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("LOCATION")
                .help("Sets your location (e.g. Dallas, TX).")
                .required(true)
                .index(2),
        )
        .arg(
            Arg::with_name("SAVE")
                .help("Sets your save location. Default is current directory.")
                .required(false)
                .index(3),
        )
        .arg(
            Arg::with_name("DISTANCE")
                .short("d")
                .long("distance")
                .value_name("DISTANCE")
                .help("Set the search distance. Default is 10 miles.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("FILE_TYPE")
                .short("f")
                .long("file_type")
                .value_name("FILE_TYPE")
                .help("Set the output file_type. Default is csv. Options: csv, json. ")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("CONCURRENCY")
                .short("c")
                .long("concurrency")
                .value_name("CONCURRENCY")
                .help("Set request concurrency. Default is 2.")
                .takes_value(true),
        )
        .get_matches();

    let query = matches.value_of("QUERY").expect("Unable to parse query");
    let location = matches
        .value_of("LOCATION")
        .expect("Unable to parse location");
    let _save = matches.value_of("SAVE").unwrap_or(".");
    let distance = matches.value_of("DISTANCE").unwrap_or("10");
    let _file_type = matches.value_of("FILE_TYPE").unwrap_or("csv");
    let concurrent_requests = matches.value_of("CONCURRENCY").unwrap_or("2");
    let start_point = geocode::geocode(&location);
    println!(
        "Found coordinates for {}: {}, {}",
        location,
        start_point.lat(),
        start_point.lng()
    );
    println!("Searching for {} within {} miles.", query, distance);
    let now = Instant::now();
    ddg::query(
        &query,
        &start_point,
        distance.parse::<f64>().unwrap(),
        concurrent_requests.parse::<usize>().unwrap(),
    );
    println!("Completed query in {} second.", now.elapsed().as_secs());
}
