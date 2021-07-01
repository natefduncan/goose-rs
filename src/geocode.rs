use geo::Point;
use geocoding::{Forward, Openstreetmap};

pub fn geocode(q: &str) -> Point<f64> {
    let osm = Openstreetmap::new();
    let res = osm.forward(&q);
    let output = res.unwrap()[0];
    Point::from((output.x(), output.y()))
}
