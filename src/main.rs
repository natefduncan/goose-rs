mod grid; 
use geo::{Point};


fn main() {
    let dallas: Point<f64> = (-96.80667, 32.78306).into();
    let austin: Point<f64> = (-97.74306, 30.26715).into();
    println!("Miles: {}", grid::miles(&dallas, &austin)); 
}
