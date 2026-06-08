use anyhow::Result;
use clap::Parser;

mod ddg;
mod files;
mod geocode;
mod grid;

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

    /// Set request concurrency.
    #[arg(short, long, default_value_t = 1usize)]
    concurrency: usize,
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

    let futures: Vec<_> = queries
        .iter()
        .map(|q| {
            let q = q.clone();
            let sp = start_point.clone();
            let distance = args.distance;
            let concurrency = args.concurrency;
            async move {
                let results = ddg::query(&q, &sp, distance, concurrency).await;
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

    if args.file_type == "csv" {
        files::output_as_csv(data)?;
    } else {
        files::output_as_json(data)?;
    }

    Ok(())
}
