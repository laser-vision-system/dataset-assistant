//! Build a csv file that describes a dataset.

/// Common functions across CSV builder variants.
mod common;
pub mod legacy;
pub mod standard;

use std::{fs, path::PathBuf};

use crate::runner::{Result, Runner, RunnerError};

type FileWriter = csv::Writer<fs::File>;

#[expect(dead_code, reason = "May use variants later")]
/// Publically announce available `CSVBuilder` versions.
#[derive(Debug)]
pub enum Version {
	Legacy,
	Standard,
}

impl Version {
	pub fn try_new(id: u8, path: PathBuf) -> Result<Box<dyn Runner>> {
		match id {
			0 => Ok(Box::new(legacy::Builder::new(path))),
			1 => Ok(Box::new(standard::Builder::new(path))),
			_ => {
				Err(RunnerError::new("invalid id passed to version new method"))
			},
		}
	}
}
