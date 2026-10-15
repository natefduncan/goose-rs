use anyhow::Result;
use clap::Parser;
use pace::Pacer;
use std::sync::{Arc, Mutex};
use std::time::Instant;

mod ddg;
mod files;
mod geocode;
mod grid;
mod pace;

#[derive(Parser)]
#[command(
    name = "Goose",
    version = "0.1.1",
    author = "Nate D.",
    about = "Query Duck Duck Go to get location data."
)]
struct Args {
    /// Sets your search value (e.g. Restaurant, Park, etc). Multiple queries in the same location can be separated by comma.
    query: String,

    /// Sets your location (e.g. Dallas, TX).
    location: String,

    /// Set the search distance in miles.
    #[arg(short, long, default_value_t = 10.0)]
    distance: f64,

    /// Set the output file type.
    #[arg(short = 'f', long = "file-type", value_parser = ["csv", "json"], default_value = "json")]
    file_type: String,

    /// Maximum requests in flight. Request *rate* adapts to DDG's limit automatically.
    #[arg(short, long, default_value_t = 16usize)]
    concurrency: usize,

    /// Never send more than this many requests per second.
    #[arg(long, default_value_t = 20.0)]
    max_rate: f64,

    /// Smallest cell (miles) to subdivide to when a cell returns a full page.
    #[arg(long, default_value_t = 0.25)]
    min_cell: f64,
}

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let args = Args::parse();

    let queries: Vec<String> = if args.query.contains(',') {
        args.query.split(',').map(|s| s.to_string()).collect()
    } else {
        vec![args.query.clone()]
    };

    let start_point = geocode::geocode(&args.location)?;

    // One pacer for all queries so they share the same rate limit.
    let pacer = Arc::new(Mutex::new(Pacer::new(2.0, args.max_rate, Instant::now())));
    let mut all_results = Vec::new();
    for q in queries {
        let results = ddg::query(&q, &start_point, args.distance, args.concurrency, args.min_cell, &pacer).await;
        all_results.push((q, results));
    }

    let mut data: Vec<ddg::Place> = vec![];
    for (query_term, temp) in all_results {
        for mut t in temp {
            t.search = Some(query_term.clone());
            data.push(t);
        }
    }

    if args.file_type == "csv" {
        files::output_as_csv(data)?;
    } else {
        files::output_as_json(data)?;
    }

    Ok(())
}
