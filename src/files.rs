use super::ddg;
use anyhow::Result;
use csv;
use serde_json;
use std::io::{self, Write};

pub fn output_as_json(data: Vec<ddg::Place>) -> Result<()> {
    let mut output = String::new();
    output.push_str("[");
    let mut is_first: u32 = 1;
    for place in data {
        let string = serde_json::to_string(&place)?;
        if is_first == 1 {
            is_first = 0;
        } else {
            output.push_str(",");
        }
        output.push_str(&string);
    }
    output.push_str("]");
    io::stdout().write_all(output.as_bytes())?;
    Ok(())
}

pub fn output_as_csv(data: Vec<ddg::Place>) -> Result<()> {
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
        "search",
    ])?;
    for p in data {
        wtr.serialize(p)?;
    }
    wtr.flush()?;
    Ok(())
}
