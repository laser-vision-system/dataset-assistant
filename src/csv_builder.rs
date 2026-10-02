//! Build a csv file that describes a dataset.

/// Common functions across CSV builder variants.
pub mod legacy;
pub mod standard;

use std::{
	fs,
	path::{Path, PathBuf},
};

use walkdir::DirEntry;

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

/// Checks that the directory entry is not a `.git` folder.
pub fn is_not_git_dir(entry: &DirEntry) -> bool {
	entry.file_name() != ".git"
}

/// Checks if a given file entry is a PNG image.
pub fn is_image_file(entry: &DirEntry) -> bool {
	!entry.file_type().is_dir()
		&& entry.path().extension().and_then(|ext| ext.to_str()) == Some("png")
}

/// Extracts the file or directory name as a valid [`String`].
pub fn get_file_name(path: &Path) -> Result<String> {
	path.file_name()
		.and_then(|name| name.to_str())
		.map(String::from)
		.ok_or_else(|| RunnerError::new("invalid unicode in file name"))
}

/// Calculates the relative path from the root directory to a given file.
pub fn get_relative_path(root: &Path, file: &Path) -> Result<String> {
	file.strip_prefix(root)
		.map_err(|why| {
			RunnerError::new(format!("failed to strip prefix: {why}"))
		})?
		.to_str()
		.ok_or_else(|| RunnerError::new("invalid unicode in relative path"))
		.map(String::from)
}
