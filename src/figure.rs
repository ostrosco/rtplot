use crate::renderer::{Renderer, Vertex};
use crate::utils;
use cgmath::Point2;
use itertools_num::linspace;
use num::Complex;
use slice_deque::SliceDeque;

#[derive(Copy, Clone, Debug)]
pub enum PlotType {
    Line,
    Dot,
}

impl Default for PlotType {
    fn default() -> Self {
        PlotType::Dot
    }
}

#[derive(Clone, Default)]
pub struct FigureConfig<'a> {
    // The min and max bounds of the x axis. If set to None, x-axis will be
    // autoscaled. Defaults to None.
    pub xlim: Option<[f32; 2]>,

    // The min and max bounds of the y axis. If set to None, y-axis will be
    // autoscaled. Defaults to None.
    pub ylim: Option<[f32; 2]>,

    // A label for the x-axis. Defaults to None.
    pub xlabel: Option<&'a str>,

    // A label for the y-axis. Defaults to None.
    pub ylabel: Option<&'a str>,

    // The color of points or lines to be drawn onto the graph. Defaults to
    // 0x000000, or black.
    pub color: [u8; 3],

    // The number of points. Defaults to 0. Set this by calling
    // Figure::init_renderer().
    pub num_points: usize,

    // The type of plot to draw. Defaults to a dot plot.
    pub plot_type: PlotType,
}

#[derive(Default)]
/// Creates a figure that will wait to receive samples, then draw them onto the
/// plot.
pub struct Figure<'a> {
    pub renderer: Option<Renderer<'a>>,
    pub config: FigureConfig<'a>,
    // A queue holding samples if the figure is going to be used for streaming
    // plotting. Size is capped at `config.num_points`.
    pub samples: SliceDeque<f32>,

    // A queue holding complex samples as above.
    pub complex_samples: SliceDeque<Complex<f32>>,
}

impl<'a> Figure<'a> {
    /// Create a figure with default settings.
    pub fn new() -> Self {
        Self {
            renderer: None,
            config: FigureConfig::default(),
            samples: SliceDeque::new(),
            complex_samples: SliceDeque::new(),
        }
    }

    /// Create a figure from an existing configuration. Useful if you don't
    /// want to use the builder pattern to initialize a figure from scratch.
    pub fn new_with_config(config: FigureConfig<'a>) -> Self {
        Self {
            renderer: None,
            config,
            samples: SliceDeque::new(),
            complex_samples: SliceDeque::new(),
        }
    }

    /// Initializes the renderer. Must be called before plotting.
    ///
    /// As nothing is Send, this is used to initialize the renderer in the
    /// thread once you make the object.
    pub fn init_renderer(mut self, num_points: usize) -> Self {
        self.renderer = Some(Renderer::new());
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

    /// Returns if the escape key has been pressed.
    ///
    /// As the event loop is not thread safe, checking for a keypress has to be
    /// done in the main thread. This is a helper function for users who choose
    /// to have the Escape key close the thread.
    ///
    pub fn handle_events(&mut self) -> bool {
        let mut status = true;

        let events_loop = match self.renderer {
            Some(ref mut rend) => &mut rend.events_loop,
            None => panic!("uninitialized renderer"),
        };

        events_loop.poll_events(|event| {
            use glium::glutin::{Event, WindowEvent};
            #[allow(clippy::single_match)]
            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::Destroyed | WindowEvent::CloseRequested => {
                        status = false
                    }
                    _ => (),
                },
                _ => (),
            }
        });
        status
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

    fn plot(&mut self, points: &[Point2<f32>]) {
        let vertices = self.normalize(&points);
        match self.renderer {
            Some(ref mut render) => {
                render.draw(&vertices, &self.config);
            }
            None => panic!("Uninitialized renderer for figure"),
        }
    }

    /// Take an array of 2D points and draw them to the plot. This overrides
    /// any samples in the queue.
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

    /// Takes a series of real samples and draws them onto the plot. This
    /// overrides any samples in the queue. The x-axis will be interpolated.
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

    /// Takes a series of real samples and draws them onto the plot. Samples
    /// received from the stream are appended to the queue and any samples
    /// exceeding the queue size are removed. The x-axis will be interpolated.
    pub fn plot_stream<T>(&mut self, y_coords: &[T])
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

    /// Takes a slice of complex samples and draws them onto the plot. Samples
    /// received from the stream are appended to the queue and any samples
    /// exceeding the queue size are removed.
    pub fn plot_complex_stream<T>(&mut self, points: &[Complex<T>])
    where
        T: Into<f32> + Copy,
    {
        if self.complex_samples.len() >= self.config.num_points + points.len() {
            for _ in 0..self.complex_samples.len() - self.config.num_points
                + points.len()
            {
                self.complex_samples.pop_front();
            }
        }

        let points: Vec<Complex<f32>> = points
            .iter()
            .map(|x| Complex::new(x.re.into(), x.im.into()))
            .collect();
        for point in points {
            self.complex_samples.push_back(point);
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

    /// Takes a slice of complex samples and draws them onto the plot. This
    /// overrides any existing samples in the queue.
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

    /// Hijacks the current thread to run the plotting and event loop.
    pub fn display(figure: &mut Figure, mut plot_fn: impl FnMut(&mut Figure)) {
        let mut status = true;
        loop {
            if !status {
                break;
            }
            status = figure.handle_events();
            plot_fn(figure);
        }
    }
}
