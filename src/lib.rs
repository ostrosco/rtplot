//! A library for creating streaming plots: where data is passed in
//! periodically and the plot automatically updates.
//!

mod figure;
mod utils;
mod window;

pub use figure::{Figure, FigureConfig, PlotType};
