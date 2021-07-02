use super::ddg;
use csv;
use std::fs::File;
use std::str; 
use std::io::{Write}; 
use serde_json; 

pub fn output_to_json(data : Vec<ddg::Place>, file_name : &str) -> Result<(), std::io::Error> {
    //JSON outfile
    let mut file = File::create(&file_name).expect("Failed to create outfile."); 
    //Start of JSON file. 
    file.write("[".as_bytes()).expect("Could not write to file.");
    let mut is_first: u32 = 1;
    for place in data {
        let string = serde_json::to_string(&place).unwrap();
        if is_first == 1 {
            is_first = 0;
        } else {
            //Write comma if not first. 
            file.write(",".as_bytes()).expect("Could not write to file.");
        }
        //Write JSON string. 
        file.write(string.as_bytes()).expect("Could not write to file.");
    }
    //End of file. 
    file.write("]".as_bytes()).expect("Could not write to file.");
    Ok(())
}

pub fn output_to_csv(data : Vec<ddg::Place>, file_name : &str) -> Result<(), std::io::Error> {
    let file = File::create(&file_name).expect("Failed to create outfile."); 
    let mut wtr = csv::WriterBuilder::new()
        .has_headers(false)
        .from_writer(file);
    wtr.write_record(&[
        "address",
        "city",
        "latitude",
        "longitude",
        "display_phone",
        "engine",
        "id",
        "name",
        "phone",
    ])?;
    for p in data {
        wtr.serialize(p)?;
    }
    wtr.flush()?;
    Ok(())
}
