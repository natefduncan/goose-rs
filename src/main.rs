mod grid; 
use geo::{Point};


fn main() {
    //url = f"https://duckduckgo.com/local.js?q={quote(q)}&tg=maps_places&rt=D&mkexp=b&is_requery=1&bbox_tl={','.join([str(i) for i in box_tl])}&bbox_br={','.join([str(i) for i in box_br])}&strict_bbox=1&wiamr=a&nyexp=b"
    let dallas: Point<f64> = (-96.80667, 32.78306).into();
    //println!("From {:?} to {:?}", &dallas, grid::get_offset(&dallas, 10., grid::Offset::North)); 
    println!("Grid: {:?}", grid::get_grid(&dallas, 10.));
}


