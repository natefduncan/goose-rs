use super::ddg;
use csv;
use serde_json;
use std::io::{self, Write};

pub fn output_as_json(data: Vec<ddg::Place>) -> Result<(), std::io::Error> {
    //JSON output
    let mut output = String::new(); 
    //let mut file = File::create(&file_name).expect("Failed to create outfile.");
    //Start of JSON file.
    output.push_str("[");
    let mut is_first: u32 = 1;
    for place in data {
        let string = serde_json::to_string(&place).unwrap();
        if is_first == 1 {
            is_first = 0;
        } else {
            //Write comma if not first.
            output.push_str(",");
        }
        //Write JSON string.
        output.push_str(&string);
    }
    //End of file.
    output.push_str("]");
    io::stdout().write_all(output.as_bytes()).expect("could not write");
    Ok(())
}

pub fn output_as_csv(data: Vec<ddg::Place>) -> Result<(), std::io::Error> {
    //let file = File::create(&file_name).expect("Failed to create outfile.");
    let mut wtr = csv::WriterBuilder::new()
        .has_headers(false)
        .from_writer(io::stdout());
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
        "search"
    ])?;
    for p in data {
        wtr.serialize(p)?;
    }
    wtr.flush()?;
    Ok(())
}
