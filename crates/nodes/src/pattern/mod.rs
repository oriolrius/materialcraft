mod catalog;
mod evaluation;
mod pattern_stamp;
mod pattern_visit;
mod raster;
mod region;
mod shape;

pub use catalog::{definition, definitions};
pub(crate) use evaluation::cook;

#[cfg(test)]
mod tests;
