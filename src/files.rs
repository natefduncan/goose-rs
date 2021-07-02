use super::ddg::Place;
use csv;
use std::error::Error;
use std::fs::{canonicalize, remove_file, File};

fn read_json_file(file_path: &str) -> Vec<Place> {
    let file = File::open(&file_path).expect("Could not open file!");
    let json: Vec<Place> = serde_json::from_reader(file).expect("Could not parse JSON!");
    return json;
}

pub fn output_to_csv(path: &str) -> Result<(), Box<dyn Error>> {
    let mut path = canonicalize(&path).unwrap();
    path.push("output.csv");
    let file = std::fs::File::create(&path).expect("Create file failed.");
    let json = read_json_file("output.json");
    let mut wtr = csv::WriterBuilder::new()
        .has_headers(false)
        .from_writer(file);
    wtr.write_record(&["address", "city", "latitude", "longitude", "display_phone", "engine", "id", "name", "phone"])?;
    for p in json {
        wtr.serialize(p)?;
    }
    wtr.flush()?;
    remove_file("output.json").expect("Could not remove file. ");
    Ok(())
}
