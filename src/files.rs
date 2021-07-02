use super::ddg::Place;  
use csv;
use std::fs::{remove_file, File, canonicalize}; 
use std::error::Error;
use std::io::{self, BufRead, BufReader};
use super::convert; 

fn read_json_file(file_path: &str) -> Vec<Place> {
    let file = File::open(&file_path).expect("Could not open file!");
    let json : Vec<Place> = serde_json::from_reader(file).expect("Could not parse JSON!");
    return json;
}

pub fn output_to_csv(path : &str) -> Result<(), Box<dyn Error>> {
    let mut path = canonicalize(&path).unwrap(); 
    path.push("output.csv"); 
    let mut wtr = csv::Writer::from_path(&path).expect("Could not find path."); 
    let json = read_json_file("output.json"); 
    for place in json {
        wtr.serialize(place); 
    }
    wtr.flush()?;
    //remove_file("output.json").expect("Could not remove file. "); 
    Ok(())
}

// From https://github.com/BurntSushi/xsv/blob/master/src/config.rs
fn io_writer(path: Option<&str>) -> io::Result<Box<io::Write + 'static>> {
    Ok(match path {
        None => Box::new(io::stdout()),
        Some(ref p) => Box::new(File::create(p)?),
    })
}

pub fn output_to_csv2(path : &str) {
    let reader = Box::new(BufReader::new(File::open("output.json").unwrap())); 
    let unwind_on = None;  
    let flatten = true; 
    let writer = io_writer(Some("output.csv")).expect("IO Writer fail.");
    let fields = None; 
    convert::write_json_to_csv(reader, writer, fields, flatten, unwind_on).expect("Write to JSON failed."); 
}
