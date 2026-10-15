//Lat = Y Long = X
use geo::algorithm::haversine_destination::HaversineDestination;
use geo::{Coordinate, Point};

enum Offset {
    Up,
    Down,
    Right,
    Left,
}

/// Split the square of side `grid_length` miles centred on `center_point` into
/// an n x n tiling, n = ceil(grid_length / rect_length). Cells share edges, so
/// there are no gaps or overlaps, and none is larger than `rect_length`.
/// Returns [top-left, bottom-right] per cell (x = longitude, y = latitude),
/// ordered north to south, west to east.
pub fn get_grids(
    center_point: &Point<f64>,
    grid_length: f64,
    rect_length: f64,
) -> Vec<[Coordinate<f64>; 2]> {
    let half = grid_length / 2.;
    let north = get_offset(center_point, half, Offset::Up).y();
    let south = get_offset(center_point, half, Offset::Down).y();
    let west = get_offset(center_point, half, Offset::Left).x();
    let east = get_offset(center_point, half, Offset::Right).x();

    let n = ((grid_length / rect_length) - 1e-9).ceil().max(1.) as usize;
    let lat_step = (north - south) / n as f64;
    let lng_step = (east - west) / n as f64;

    let mut output = Vec::with_capacity(n * n);
    for row in 0..n {
        let top = north - lat_step * row as f64;
        for col in 0..n {
            let left = west + lng_step * col as f64;
            output.push([
                Coordinate { x: left, y: top },
                Coordinate { x: left + lng_step, y: top - lat_step },
            ]);
        }
    }
    output
}

fn get_offset(start_point: &Point<f64>, miles: f64, offset: Offset) -> Point<f64> {
    let meters = miles * 1609.344;
    let bearing = match offset {
        Offset::Up => 0.0,
        Offset::Right => 90.0,
        Offset::Down => 180.0,
        Offset::Left => 270.0,
    };
    start_point.haversine_destination(bearing, meters)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn centre() -> Point<f64> {
        Point::new(-96.765517, 32.814257)
    }

    fn miles(a: &Point<f64>, b: &Point<f64>) -> f64 {
        use geo::algorithm::haversine_distance::HaversineDistance;
        a.haversine_distance(b) / 1609.344
    }

    #[test]
    fn cell_count_is_ceil_squared() {
        for (g, r, n) in [(10., 5., 2), (3., 1.5, 2), (7., 3.5, 2), (10., 3., 4), (1., 1., 1), (60., 20., 3)] {
            assert_eq!(get_grids(&centre(), g, r).len(), n * n, "grid {g} cell {r}");
        }
    }

    #[test]
    fn cells_tile_the_square_exactly() {
        let cells = get_grids(&centre(), 10., 3.);
        let (mut w, mut e, mut s, mut n) = (f64::MAX, f64::MIN, f64::MAX, f64::MIN);
        for c in &cells {
            assert!(c[0].x < c[1].x && c[0].y > c[1].y);
            w = w.min(c[0].x);
            e = e.max(c[1].x);
            n = n.max(c[0].y);
            s = s.min(c[1].y);
        }
        // Spans the requested square, centred on the centre point.
        let width = miles(&Point::new(w, centre().y()), &Point::new(e, centre().y()));
        let height = miles(&Point::new(centre().x(), s), &Point::new(centre().x(), n));
        assert!((width - 10.).abs() < 0.01 && (height - 10.).abs() < 0.01, "{width} x {height}");
        assert!((((w + e) / 2.) - centre().x()).abs() < 1e-4);
        assert!((((n + s) / 2.) - centre().y()).abs() < 1e-4);
        // Total area equals the bounding box: no gaps, no overlaps.
        let area: f64 = cells.iter().map(|c| (c[1].x - c[0].x) * (c[0].y - c[1].y)).sum();
        assert!((area - (e - w) * (n - s)).abs() < 1e-12);
        // Every cell is at most the requested size.
        for c in &cells {
            let h = miles(&Point::new(c[0].x, c[0].y), &Point::new(c[0].x, c[1].y));
            assert!(h <= 3.0 + 1e-6, "{h}");
        }
    }

    #[test]
    fn adjacent_cells_share_edges() {
        let cells = get_grids(&centre(), 10., 5.);
        assert_eq!(cells[0][1].x, cells[1][0].x);
        assert_eq!(cells[0][1].y, cells[3][0].y);
    }
}
