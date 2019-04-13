use crate::renderer::{Renderer, Vertex};
use std::marker::PhantomData;
use crate::utils::{self, Point2D};
use itertools_num::linspace;
use num::Complex;

#[derive(Default)]
pub struct FigureConfig<'a> {
    pub xlim: Option<[f32; 2]>,
    pub ylim: Option<[f32; 2]>,
    pub xlabel: Option<&'a str>,
    pub ylabel: Option<&'a str>,
}

pub struct Figure<'a, T>
where
    T: Into<f32> + Copy,
{
    pub renderer: Option<Renderer<'a>>,
    pub config: FigureConfig<'a>,
    _phantom: PhantomData<T>,
}

impl<'a, T> Figure<'a, T>
where
    T: Into<f32> + Copy,
{
    pub fn new() -> Self {
        Figure {
            renderer: None,
            config: FigureConfig::default(),
            _phantom: PhantomData,
        }

    }

    pub fn init_renderer(mut self) -> Self {
        self.renderer = Some(Renderer::new());
        self
    }

    /// Sets the x min and max limits for plotting.
    pub fn xlim(mut self, xlim: [f32; 2]) -> Self {
        self.config.xlim = Some(xlim);
        self
    }

    /// Sets the y min and max limits for plotting.
    pub fn ylim(mut self, ylim: [f32; 2]) -> Self {
        self.config.ylim = Some(ylim);
        self
    }

    pub fn xlabel(mut self, xlabel: &'a str) -> Self {
        self.config.xlabel = Some(xlabel);
        self
    }

    pub fn ylabel(mut self, ylabel: &'a str) -> Self {
        self.config.ylabel = Some(ylabel);
        self
    }

    fn normalize(&self, points: &[Point2D]) -> Vec<Vertex>
    where
        T: Into<f32> + Copy,
    {
        let [min_x, max_x] = match self.config.xlim {
            Some(lim) => lim,
            None => utils::calc_xlims(points),
        };
        let [min_y, max_y] = match self.config.ylim {
            Some(lim) => lim,
            None => utils::calc_ylims(points),
        };
        let mut vertices = vec![];
        for point in points {
            let x = if max_x != min_x {
                1.5 * (point.x - min_x) / (max_x - min_x) - 0.75
            } else {
                1.5 * point.x - 0.75
            };
            let y = if max_y != min_y {
                1.5 * (point.y - min_y) / (max_y - min_y) - 0.75
            } else {
                1.5 * point.y - 0.75
            };
            vertices.push(Vertex::new(x, y));
        }
        vertices
    }

    pub fn plot(&mut self, points: &[Point2D]) {
        let vertices = self.normalize(&points);
        match self.renderer {
            Some(ref mut render) => {
                render.draw(&vertices, &self.config);
            }
            None => panic!("Uninitialized renderer for figure"),
        }
    }

    /// Take an array of points and draw it to the screen.
    pub fn plot_xy(&mut self, points: &[(T, T)])
    where
        T: Into<f32> + Copy,
    {
        let points: Vec<Point2D> = points.iter().map(|pt| pt.into()).collect();
        self.plot(&points);
    }

    /// Take an array of y coordinates, interpolate the x, and then plot.
    pub fn plot_y(&mut self, y_coords: &[T])
    where
        T: Into<f32> + Copy,
    {
        let x_coords = linspace(-0.5f32, 0.5f32, y_coords.len());
        let points: Vec<Point2D> = x_coords
            .zip(y_coords.iter())
            .map(|(x, y)| Point2D::new(x, (*y).into()))
            .collect();
        self.plot(&points);
    }

    pub fn plot_complex(&mut self, coords: &[Complex<T>])
    where
        T: Into<f32> + Copy,
    {
        let points: Vec<Point2D> =
            coords.iter().map(|pt| (pt.re, pt.im).into()).collect();
        self.plot(&points);
    }
}
