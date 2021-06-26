//Lat = Y Long = X
use geo::{Point, Rect, Coordinate};
use geo::algorithm::geodesic_distance::GeodesicDistance; 

pub enum Offset {
    North, 
    South,
    East, 
    West
}

fn miles(p1 : &Point<f64>, p2 : &Point<f64>) -> f64 {
    let meters_to_miles : f64 = 0.00062137119223733; 
    let meters = p1.geodesic_distance(&p2); 
    let miles : f64 = meters_to_miles * meters; 
    return miles
}

pub fn get_grid(center_point : &Point<f64>, miles : f64) -> Rect<f64> {
    let bottom_center : Point<f64> = get_offset(center_point, miles / 2., Offset::South); 
    let bottom_left : Point<f64> = get_offset(&bottom_center, miles / 2., Offset::West); 
    let top_left : Point<f64> = get_offset(&bottom_left, miles, Offset::North); 
    let top_right : Point<f64> = get_offset(&top_left,  miles, Offset::East); 
    let bl_coord : Coordinate<f64> = bottom_left.into(); 
    let tr_coord : Coordinate<f64> = top_right.into(); 
    Rect::new(
        bl_coord,
        tr_coord
    )
}

fn get_offset(start_point : &Point<f64>, miles : f64, offset : Offset) -> Point<f64> {
    let miles_to_degrees : f64 = 64.52626802; 
    let degrees = miles / miles_to_degrees; 
    let mut lat_translate : f64 = 0. ; 
    let mut lng_translate : f64 = 0. ; 
    match offset {
        Offset::North => {lat_translate = lat_translate + degrees} 
        Offset::South => {lat_translate = lat_translate - degrees;}
        Offset::East => {lng_translate = lng_translate + degrees;}
        Offset::West => {lng_translate = lat_translate - degrees;}
    }
    let output : Point<f64> = (start_point.lng() + lng_translate, start_point.lat() + lat_translate).into();
    return output
}

/*
pub fn get_rect(center_point : &Point<f64>, length_miles : f64) -> Rect<f64> {
    center_point.lat(); 
    center_point.lng(); 
}*/

//1. Select start point. 
//2. Select distance from center. 
//3. Generate boxes.