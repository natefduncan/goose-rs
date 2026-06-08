use anyhow::{anyhow, Result};
use geo::Point;
use geocoding::{Forward, Openstreetmap};

pub fn geocode(q: &str) -> Result<Point<f64>> {
    let osm = Openstreetmap::new();
    let res = osm.forward(q).map_err(|e| anyhow!("Geocoding request failed: {}", e))?;
    if res.is_empty() {
        return Err(anyhow!("Location '{}' not found", q));
    }
    let output = res[0];
    Ok(Point::from((output.x(), output.y())))
}
