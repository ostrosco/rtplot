use cgmath::Point2;
pub struct Point2D {
    pub x: f32,
    pub y: f32,
}

impl Point2D {
    pub fn new(x: f32, y: f32) -> Self {
        Point2D { x, y }
    }
}

impl<T> From<(T, T)> for Point2D
where
    T: Into<f32> + Copy,
{
    fn from(val: (T, T)) -> Self {
        Point2D::new(val.0.into(), val.1.into())
    }
}

impl<T> From<&(T, T)> for Point2D
where
    T: Into<f32> + Copy,
{
    fn from(val: &(T, T)) -> Self {
        Point2D::new(val.0.into(), val.1.into())
    }
}

fn calc_min_max(points: &[f32]) -> [f32; 2] {
    let min_val = points
        .iter()
        .min_by(|x, y| x.partial_cmp(y).unwrap())
        .unwrap();
    let max_val = points
        .iter()
        .max_by(|x, y| x.partial_cmp(y).unwrap())
        .unwrap();
    [*min_val, *max_val]
}

pub fn calc_xlims(points: &[Point2<f32>]) -> [f32; 2] {
    let x: Vec<f32> = points.iter().map(|pt| pt.x).collect();
    let xlims: [f32; 2] = calc_min_max(&x);
    xlims
}

pub fn calc_ylims(points: &[Point2<f32>]) -> [f32; 2] {
    let y: Vec<f32> = points.iter().map(|pt| pt.y).collect();
    let ylims: [f32; 2] = calc_min_max(&y);
    ylims
}
