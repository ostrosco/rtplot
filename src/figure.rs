use crate::utils;
use crate::window::{Vertex, Window};
use cgmath::Point2;
use glium::glutin::platform::desktop::EventLoopExtDesktop;
use itertools_num::linspace;
use num::Complex;
use slice_deque::SliceDeque;

#[derive(Copy, Clone, Debug)]
pub enum PlotType {
    /// Draws a continuous line between points.
    Line,

    /// Each point is drawn as a small diamond.
    Dot,
}

impl Default for PlotType {
    fn default() -> Self {
        PlotType::Dot
    }
}

#[derive(Clone, Default)]
pub struct FigureConfig<'a> {
    /// The min and max bounds of the x axis. If set to None, x-axis will be
    /// autoscaled. Defaults to None.
    pub xlim: Option<[f32; 2]>,

    /// The min and max bounds of the y axis. If set to None, y-axis will be
    /// autoscaled. Defaults to None.
    pub ylim: Option<[f32; 2]>,

    /// A label for the x-axis. Defaults to None.
    pub xlabel: Option<&'a str>,

    /// A label for the y-axis. Defaults to None.
    pub ylabel: Option<&'a str>,

    /// The color of points or lines to be drawn onto the graph. Defaults to
    /// 0x000000, or black.
    pub color: [u8; 3],

    /// The type of plot to draw. Defaults to a dot plot.
    pub plot_type: PlotType,
}

#[derive(Default)]
/// Creates a figure that will wait to receive samples, then draw them onto the
/// plot.
pub struct Figure<'a> {
    window: Window<'a>,
    config: FigureConfig<'a>,

    /// A queue holding samples if the figure is going to be used for streaming
    /// plotting. Size is capped at `queue_size`.
    samples: SliceDeque<f32>,

    /// A queue holding complex samples as above.
    complex_samples: SliceDeque<Complex<f32>>,

    /// The number of points. Defaults to 0.
    queue_size: usize,

    /// Indicates whether the x axis is dynamic.
    x_dynamic: bool,

    /// Indicates whether the y axis is dynamic.
    y_dynamic: bool,
}

impl<'a> Figure<'a> {
    /// Create a figure with default settings.
    pub fn new(queue_size: usize) -> Self {
        Self {
            window: Window::new(),
            config: FigureConfig::default(),
            samples: SliceDeque::new(),
            complex_samples: SliceDeque::new(),
            queue_size,
            x_dynamic: true,
            y_dynamic: true,
        }
    }

    /// Create a figure from an existing configuration. Useful if you don't
    /// want to use the builder pattern to initialize a figure from scratch.
    pub fn new_with_config(config: FigureConfig<'a>, queue_size: usize) -> Self {
        let x_dynamic = config.xlim.is_none();
        let y_dynamic = config.ylim.is_none();
        Self {
            window: Window::new(),
            config,
            samples: SliceDeque::new(),
            complex_samples: SliceDeque::new(),
            queue_size,
            x_dynamic,
            y_dynamic,
        }
    }

    /// Sets the x min and max limits for plotting.
    pub fn xlim(mut self, xlim: [f32; 2]) -> Self {
        self.config.xlim = Some(xlim);
        self.x_dynamic = false;
        self
    }

    /// Sets the y min and max limits for plotting.
    pub fn ylim(mut self, ylim: [f32; 2]) -> Self {
        self.config.ylim = Some(ylim);
        self.y_dynamic = false;
        self
    }

    /// Sets the x label to display.
    pub fn xlabel(mut self, xlabel: &'a str) -> Self {
        self.config.xlabel = Some(xlabel);
        self
    }

    /// Sets the y label to display.
    pub fn ylabel(mut self, ylabel: &'a str) -> Self {
        self.config.ylabel = Some(ylabel);
        self
    }

    /// Sets the color of the line to draw.
    pub fn color(mut self, r: u8, g: u8, b: u8) -> Self {
        self.config.color = [r, g, b];
        self
    }

    /// Sets the type of plot to generate.
    pub fn plot_type(mut self, plot_type: PlotType) -> Self {
        self.config.plot_type = plot_type;
        self
    }

    /// Checks events to see if the figure should close or not. Returns
    /// true if the window received a close event, false otherwise. In
    /// most cases, you don't need to handle events yourself; use
    /// Figure::display() instead.
    pub fn should_close_window(&mut self) -> bool {
        let mut should_close_window = false;

        let events_loop = &mut self.window.events_loop;

        events_loop.run_return(|event, _, control_flow| {
            use glium::glutin::event::{Event, WindowEvent};
            use glium::glutin::event_loop::ControlFlow;
            #[allow(clippy::single_match)]
            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::Destroyed | WindowEvent::CloseRequested => {
                        should_close_window = true
                    }
                    _ => (),
                },
                _ => (),
            }
            *control_flow = ControlFlow::Exit;
        });
        should_close_window
    }

    /// Normalizes the received points to [-0.5, 0.5] for drawing in OpenGL.
    fn normalize(&mut self, points: &[Point2<f32>]) -> Vec<Vertex> {
        let [min_x, max_x] = if self.x_dynamic {
            let xlims = utils::calc_xlims(points);
            self.config.xlim = Some(xlims);
            xlims
        } else {
            self.config.xlim.unwrap()
        };
        let [min_y, max_y] = if self.y_dynamic {
            let ylims = utils::calc_ylims(points);
            self.config.ylim = Some(ylims);
            ylims
        } else {
            self.config.ylim.unwrap()
        };
        let mut vertices = vec![];
        for point in points {
            // If there are points outside the min and max range, skip over
            // them since we won't draw them anyways.
            if point.x > max_x || point.x < min_x || point.y > max_y || point.y < min_y {
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

    /// A helper function for normalizing and drawing points to the window.
    fn plot(&mut self, points: &[Point2<f32>]) {
        let vertices = self.normalize(&points);
        self.window.draw(&vertices, &self.config);
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
        if self.samples.len() >= self.queue_size + y_coords.len() {
            for _ in 0..self.samples.len() - self.queue_size + y_coords.len() {
                self.samples.pop_front();
            }
        }
        let y: Vec<f32> = y_coords.iter().map(|y| (*y).into()).collect();
        for point in &y {
            self.samples.push_back(*point);
        }
        let x_coords = linspace(-0.5f32, 0.5f32, self.queue_size);
        let points: Vec<Point2<f32>> = x_coords
            .zip(self.samples.iter())
            .map(|(x, y)| Point2::new(x, *y))
            .collect();
        let vertices = self.normalize(&points);
        self.window.draw(&vertices, &self.config);
    }

    /// Takes a slice of complex samples and draws them onto the plot. Samples
    /// received from the stream are appended to the queue and any samples
    /// exceeding the queue size are removed.
    pub fn plot_complex_stream<T>(&mut self, points: &[Complex<T>])
    where
        T: Into<f32> + Copy,
    {
        if self.complex_samples.len() >= self.queue_size + points.len() {
            for _ in 0..self.complex_samples.len() - self.queue_size + points.len() {
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
        self.window.draw(&vertices, &self.config);
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
        while !figure.should_close_window() {
            plot_fn(figure);
        }
    }
}
