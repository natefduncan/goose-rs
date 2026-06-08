use super::ddg;
use anyhow::Result;
use csv;
use serde_json;
use std::io;

pub fn output_as_json(data: Vec<ddg::Place>) -> Result<()> {
    serde_json::to_writer(io::stdout(), &data)?;
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
