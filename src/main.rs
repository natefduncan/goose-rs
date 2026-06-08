extern crate clap;
use anyhow::{anyhow, Result};
use clap::{App, Arg};

mod ddg;
mod files;
mod geocode;
mod grid;

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let matches = App::new("Goose")
        .version("0.1.1")
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
    let queries: Vec<String> = if query.contains(',') {
        query.split(',').map(|s| s.to_string()).collect()
    } else {
        vec![query.to_string()]
    };
    let location = matches
        .value_of("LOCATION")
        .expect("Unable to parse location");
    let distance = matches.value_of("DISTANCE").unwrap_or("10");
    let file_type = matches.value_of("FILE-TYPE").unwrap_or("json");
    let concurrent_requests = matches.value_of("CONCURRENCY").unwrap_or("1");

    let distance_val = distance.parse::<f64>().map_err(|_| {
        anyhow!("Invalid --distance '{}': must be a number", distance)
    })?;
    let concurrency_val = concurrent_requests.parse::<usize>().map_err(|_| {
        anyhow!(
            "Invalid --concurrency '{}': must be a positive integer",
            concurrent_requests
        )
    })?;

    let start_point = geocode::geocode(location)?;

    let futures: Vec<_> = queries
        .iter()
        .map(|q| {
            let q = q.clone();
            let sp = start_point.clone();
            async move {
                let results = ddg::query(&q, &sp, distance_val, concurrency_val).await;
                (q, results)
            }
        })
        .collect();

    let all_results = futures::future::join_all(futures).await;

    let mut data: Vec<ddg::Place> = vec![];
    for (query_term, temp) in all_results {
        for mut t in temp {
            t.search = Some(query_term.clone());
            data.push(t);
        }
    }

    if file_type == "csv" {
        files::output_as_csv(data)?;
    } else if file_type == "json" {
        files::output_as_json(data)?;
    }

    Ok(())
}
