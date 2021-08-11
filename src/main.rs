extern crate clap;
use clap::{App, Arg};

mod ddg;
mod files;
mod geocode;
mod grid;

fn main() {
    let matches = App::new("Goose")
        .version("1.0")
        .author("Nate D.")
        .about("Query Duck Duck Go to get location data.")
        .arg(
            Arg::with_name("QUERY")
                .help("Sets your search value (e.g. Restaurant, Park, etc). Multiple queries in the same location can be separated by comma.")
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
            Arg::with_name("DISTANCE")
                .short("d")
                .long("distance")
                .value_name("DISTANCE")
                .help("Set the search distance. Default is 10 miles.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("FILE-TYPE")
                .short("f")
                .long("file-type")
                .value_name("FILE-TYPE")
                .help("Set the output file_type. Default is json. Options: csv, json. ")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("CONCURRENCY")
                .short("c")
                .long("concurrency")
                .value_name("CONCURRENCY")
                .help("Set request concurrency. Default is 1.")
                .takes_value(true),
        )
        .get_matches();

    let query = matches.value_of("QUERY").expect("Unable to parse query");
    let queries : Vec<&str>; 
    if query.contains(",") {
        queries = query.split(",").collect::<Vec<&str>>(); 
    } else {
        queries = vec![query];
    }
    let location = matches
        .value_of("LOCATION")
        .expect("Unable to parse location");
    let distance = matches.value_of("DISTANCE").unwrap_or("10");
    let file_type = matches.value_of("FILE-TYPE").unwrap_or("json");
    let concurrent_requests = matches.value_of("CONCURRENCY").unwrap_or("1");
    let start_point = geocode::geocode(&location);
    let mut data: Vec<ddg::Place> = vec![];
    for query in queries {
        let temp = ddg::query(
            &query,
            &start_point,
            distance.parse::<f64>().unwrap(),
            concurrent_requests.parse::<usize>().unwrap(),
        );
        for mut t in temp {
            t.search = Some(query.to_string()); 
            data.push(t); 
        }
    }
    if file_type == "csv" {
        files::output_as_csv(data).expect("Write to csv failed.");
    } else if file_type == "json" {
        files::output_as_json(data).expect("Write to json failed.")
    }
}
