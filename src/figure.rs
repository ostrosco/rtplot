use crate::renderer::{Renderer, Vertex};
use crate::utils;
use cgmath::Point2;
use itertools_num::linspace;
use num::Complex;
use slice_deque::SliceDeque;

pub enum PlotType {
    Line,
    Dot,
}

impl Default for PlotType {
    fn default() -> Self {
        PlotType::Dot
    }
}

#[derive(Default)]
pub struct FigureConfig<'a> {
    pub xlim: Option<[f32; 2]>,
    pub ylim: Option<[f32; 2]>,
    pub xlabel: Option<&'a str>,
    pub ylabel: Option<&'a str>,
    pub color: [u8; 3],
    pub num_points: usize,
    pub plot_type: PlotType,
}

#[derive(Default)]
pub struct Figure<'a> {
    pub renderer: Option<Renderer<'a>>,
    pub config: FigureConfig<'a>,
    pub samples: SliceDeque<f32>,
    pub complex_samples: SliceDeque<Complex<f32>>,
}

impl<'a> Figure<'a> {
    pub fn new() -> Self {
        Figure {
            renderer: None,
            config: FigureConfig::default(),
            samples: SliceDeque::new(),
            complex_samples: SliceDeque::new(),
        }
    }

    pub fn init_renderer(mut self, num_points: usize) -> Self {
        self.renderer = Some(Renderer::new(num_points));
        self.config.num_points = num_points;
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

    pub fn color(mut self, r: u8, g: u8, b: u8) -> Self {
        self.config.color = [r, g, b];
        self
    }

    pub fn plot_type(mut self, plot_type: PlotType) -> Self {
        self.config.plot_type = plot_type;
        self
    }

    fn normalize(&self, points: &[Point2<f32>]) -> Vec<Vertex> {
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
            // If there are points outside the min and max range, skip over
            // them since we won't draw them anyways.
            if point.x > max_x
                || point.x < min_x
                || point.y > max_y
                || point.y < min_y
            {
                continue;
            }
            let error: f32 = 0.0;
            let x = if (max_x - min_x).abs() > error {
                1.5 * (point.x - min_x) / (max_x - min_x) - 0.75
            } else {
                1.5 * point.x - 0.75
            };
            let y = if (max_y - min_y).abs() > error {
                1.5 * (point.y - min_y) / (max_y - min_y) - 0.75
            } else {
                1.5 * point.y - 0.75
            };
            vertices.push(Vertex::new(x, y, self.config.color));
        }
        vertices
    }

    pub fn plot(&mut self, points: &[Point2<f32>]) {
        let vertices = self.normalize(&points);
        match self.renderer {
            Some(ref mut render) => {
                render.draw(&vertices, &self.config);
            }
            None => panic!("Uninitialized renderer for figure"),
        }
    }

    /// Take an array of points and draw it to the screen.
    pub fn plot_xy<T>(&mut self, points: &[(T, T)])
    where
        T: Into<f32> + Copy,
    {
        let points: Vec<Point2<f32>> = points
            .iter()
            .map(|pt| Point2::new(pt.0.into(), pt.1.into()))
            .collect();
        self.plot(&points);
    }

    /// Take an array of y coordinates, interpolate the x, and then plot.
    pub fn plot_y<T>(&mut self, y_coords: &[T])
    where
        T: Into<f32> + Copy,
    {
        let x_coords = linspace(-0.5f32, 0.5f32, y_coords.len());
        let points: Vec<Point2<f32>> = x_coords
            .zip(y_coords.iter())
            .map(|(x, y)| Point2::new(x, (*y).into()))
            .collect();
        self.plot(&points);
    }

    pub fn plot_samples<T>(&mut self, y_coords: &[T])
    where
        T: Into<f32> + Copy,
    {
        if self.samples.len() >= self.config.num_points + y_coords.len() {
            for _ in
                0..self.samples.len() - self.config.num_points + y_coords.len()
            {
                self.samples.pop_front();
            }
        }
        let y: Vec<f32> = y_coords.iter().map(|y| (*y).into()).collect();
        for point in &y {
            self.samples.push_back(*point);
        }
        let x_coords = linspace(-0.5f32, 0.5f32, self.config.num_points);
        let points: Vec<Point2<f32>> = x_coords
            .zip(self.samples.iter())
            .map(|(x, y)| Point2::new(x, *y))
            .collect();
        let vertices = self.normalize(&points);
        match self.renderer {
            Some(ref mut render) => {
                render.draw(&vertices, &self.config);
            }
            None => panic!("Uninitialized renderer for figure"),
        }
    }

    pub fn plot_complex_samples(&mut self, points: &[Complex<f32>]) {
        if self.complex_samples.len() >= self.config.num_points + points.len() {
            for _ in 0..self.complex_samples.len() - self.config.num_points
                + points.len()
            {
                self.complex_samples.pop_front();
            }
        }
        for point in points {
            self.complex_samples.push_back(*point);
        }
        let points: Vec<Point2<f32>> = self
            .complex_samples
            .iter()
            .map(|x| Point2::new(x.re, x.im))
            .collect();
        let vertices = self.normalize(&points);
        match self.renderer {
            Some(ref mut render) => {
                render.draw(&vertices, &self.config);
            }
            None => panic!("Uninitialized renderer for figure"),
        }
    }

    pub fn plot_complex<T>(&mut self, coords: &[Complex<T>])
    where
        T: Into<f32> + Copy,
    {
        let points: Vec<Point2<f32>> = coords
            .iter()
            .map(|pt| Point2::new(pt.re.into(), pt.im.into()))
            .collect();
        self.plot(&points);
    }
}
