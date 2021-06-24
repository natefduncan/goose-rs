//Lat = Y Long = X
use geo::{Point};
use geo::algorithm::geodesic_distance::GeodesicDistance; 

pub fn miles(p1 : &Point<f64>, p2 : &Point<f64>) -> f64 {
    let meters_to_miles : f64 = 0.00062137119223733;
    let meters = p1.geodesic_distance(&p2); 
    let miles : f64 = meters_to_miles * meters; 
    return miles
}