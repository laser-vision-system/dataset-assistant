//! Build a csv file that describes a dataset.

pub mod legacy;
pub mod standard;

use std::{error::Error, fmt, fs, path::PathBuf, thread};

type _JoinHandle = thread::JoinHandle<Result<()>>;
type _FileWriter = csv::Writer<fs::File>;
pub type Result<T> = std::result::Result<T, CSVBuilderError>;

/// Implement this on a CSV builder type.
///
/// Will automatically implement the [`CSVBuilder`] trait with a blanket
/// implementation.
trait CSVBuilderHandle {
	fn handle(&mut self) -> &mut Option<_JoinHandle>;
}

#[allow(
	private_bounds,
	reason = "The supertrait is for internal implementation and usage only"
)]
/// Front-facing trait for types that build a CSV file.
pub trait CSVBuilder: CSVBuilderHandle {
	fn is_done(&mut self) -> Option<CSVBuilderResult> {
		let Some(handle) = self.handle().take() else {
			return None;
		};

		if handle.is_finished() {
			return Some(handle.into());
		} else {
			self.handle().replace(handle);
			None
		}
	}
}

impl<T> CSVBuilder for T where T: CSVBuilderHandle {}

#[allow(dead_code, reason = "May use later")]
/// Publically announce available CSVBuilder versions.
#[derive(Debug)]
pub enum Version {
	Legacy,
	Standard,
}

impl Version {
	pub fn try_new(id: u8, path: PathBuf) -> Result<Box<dyn CSVBuilder>> {
		match id {
			0 => Ok(Box::new(legacy::Builder::new(path))),
			1 => Ok(Box::new(standard::Builder::new(path))),
			_ => Err(CSVBuilderError::new(
				"invalid id passed to version new method",
			)),
		}
	}
}

#[derive(Debug)]
pub struct CSVBuilderResult {
	handle: _JoinHandle,
}

impl CSVBuilderResult {
	pub fn join(self) -> Result<()> {
		self.handle
			.join()
			.expect("thread panicked and I don't know how to handle it")
	}
}

impl From<_JoinHandle> for CSVBuilderResult {
	fn from(handle: _JoinHandle) -> Self {
		Self { handle }
	}
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct CSVBuilderError {
	why: String,
}

impl CSVBuilderError {
	fn new(why: impl Into<String>) -> Self {
		Self { why: why.into() }
	}
}

impl fmt::Display for CSVBuilderError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "CSVBuilderError: {}", self.why)
	}
}

impl Error for CSVBuilderError {}

/// Common functions across CSV builder variants.
pub(super) mod common {
	use std::path::Path;

	use walkdir::DirEntry;

	use super::{CSVBuilderError, Result};

	/// Checks that the directory entry is not a `.git` folder.
	pub(super) fn is_not_git_dir(entry: &DirEntry) -> bool {
		entry.file_name() != ".git"
	}

	/// Checks if a given file entry is a PNG image.
	pub(super) fn is_image_file(entry: &DirEntry) -> bool {
		entry.file_type().is_file()
			&& entry.path().extension().and_then(|ext| ext.to_str())
				== Some("png")
	}

	/// Extracts the file or directory name as a valid [`String`].
	pub(super) fn get_file_name(path: &Path) -> Result<String> {
		path.file_name()
			.and_then(|name| name.to_str())
			.map(String::from)
			.ok_or_else(|| CSVBuilderError::new("invalid unicode in file name"))
	}

	/// Calculates the relative path from the root directory to a given file.
	pub(super) fn get_relative_path(
		root: &Path,
		file: &Path,
	) -> Result<String> {
		file.strip_prefix(root)
			.map_err(|why| {
				CSVBuilderError::new(format!("failed to strip prefix: {why}"))
			})?
			.to_str()
			.ok_or_else(|| {
				CSVBuilderError::new("invalid unicode in relative path")
			})
			.map(String::from)
	}
}
