//Lat = Y Long = X
use geo::algorithm::geodesic_distance::GeodesicDistance;
use geo::{Coordinate, Point, Rect};

enum Offset {
    Up,
    Down,
    Right,
    Left,
}

fn round_float(float: f64, decimal_places: i32) -> f64 {
    let mut mult: i32 = 1;
    for _ in 1..decimal_places {
        mult = mult * 10
    }
    let mult_float = f64::from(mult);
    return (float * mult_float).round() / mult_float;
}

fn _miles(p1: &Point<f64>, p2: &Point<f64>) -> f64 {
    let meters_to_miles: f64 = 0.00062137119223733;
    let meters = p1.geodesic_distance(&p2);
    let miles: f64 = meters_to_miles * meters;
    return miles;
}

fn tl_br_from_center(center_point: &Point<f64>, miles: f64) -> [Coordinate<f64>; 2] {
    let bottom_center: Point<f64> = get_offset(center_point, miles / 2., Offset::Down);
    let bottom_left: Point<f64> = get_offset(&bottom_center, miles / 2., Offset::Left);
    let bottom_right: Point<f64> = get_offset(&bottom_left, miles, Offset::Right);
    let top_left: Point<f64> = get_offset(&bottom_left, miles, Offset::Up);
    let br_coord: Coordinate<f64> = bottom_right.into();
    let tl_coord: Coordinate<f64> = top_left.into();
    [tl_coord, br_coord]
}

fn rect_from_center(center_point: &Point<f64>, miles: f64) -> Rect<f64> {
    let tl_br = tl_br_from_center(&center_point, miles);
    Rect::new(tl_br[0], tl_br[1])
}

pub fn get_grids(
    center_point: &Point<f64>,
    grid_length: f64,
    rect_length: f64,
) -> Vec<[Coordinate<f64>; 2]> {
    let mut output = Vec::new();
    // find top left start point.
    let left = get_offset(
        &center_point,
        (grid_length / 2.) - (rect_length / 2.),
        Offset::Left,
    );
    let mut temp_point = get_offset(&left, (grid_length / 2.) - (rect_length / 2.), Offset::Up);
    let upper_bound_lng = round_float(
        get_offset(&temp_point, grid_length - (rect_length / 2.), Offset::Right).lng(),
        5,
    ); //Y Axis
    let lower_bound_lat = round_float(
        get_offset(&temp_point, grid_length - (rect_length / 2.), Offset::Down).lat(),
        5,
    ); //X Axis

    let mut temp_rect: Rect<f64>;
    loop {
        //Check if exceeded lat
        if round_float(temp_point.lat(), 5) <= lower_bound_lat {
            break;
        }
        temp_rect = rect_from_center(&temp_point, rect_length);
        //Check if exceeded lng
        if round_float(temp_rect.max().x, 5) >= upper_bound_lng {
            temp_point = get_offset(&temp_point, grid_length - rect_length, Offset::Left);
            temp_point = get_offset(&temp_point, rect_length, Offset::Down);
        } else {
            temp_point = get_offset(&temp_point, rect_length, Offset::Right);
        }
        output.push(tl_br_from_center(&temp_point, rect_length));
    }
    return output;
}

fn get_offset(start_point: &Point<f64>, miles: f64, offset: Offset) -> Point<f64> {
    let miles_to_degrees: f64 = 64.52626802;
    let degrees = miles / miles_to_degrees;
    let mut lat_translate: f64 = 0.;
    let mut lng_translate: f64 = 0.;
    match offset {
        Offset::Up => lat_translate = lat_translate + degrees,
        Offset::Down => {
            lat_translate = lat_translate - degrees;
        }
        Offset::Right => {
            lng_translate = lng_translate + degrees;
        }
        Offset::Left => {
            lng_translate = lat_translate - degrees;
        }
    }
    let output: Point<f64> = (
        start_point.lng() + lng_translate,
        start_point.lat() + lat_translate,
    )
        .into();
    return output;
}
