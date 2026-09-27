//! The local filesystem path where the dataset is stored.
//!
//! The dataset primarily consists of a root directory. The root directory
//! should remain fixed for the duration of the collection process. The actual
//! location that the images will be stored at will vary by a position and
//! intensity, represented as nested subfolders of the root directory.
//!
//! The [`Dataset`] should never be consumed. Clearing the position after
//! running the collection procedure is inconvenient. Clearing the intensity
//! is also unnecessary.
//!
//! It would be nice to use [`std::fs::Dir`], but it is current
//! experimental-only. If anyone is working on this in the future, consider
//! refactoring to use it.

use std::{
	fmt::Display,
	fs, io,
	path::{Path, PathBuf},
};

use eframe::egui::TextBuffer as _;

use super::subset::Subset;

/// Query a path for availablility information.
fn is_path_available(path: &Path) -> io::Result<()> {
	// If the path does not exist, it must be empty.
	if !path.try_exists()? {
		return Ok(());
	}
	if !path.is_dir() {
		return Err(io::Error::from(io::ErrorKind::NotADirectory));
	}
	// This is not technically an error, but it provides information.
	if path.read_dir()?.count() > 0 {
		return Err(io::Error::from(io::ErrorKind::DirectoryNotEmpty));
	}
	Ok(())
}

/// A dataset is a large collection of images that will be used for training a
/// machine learning model.
///
/// Does not support any kind of dynamic adjustment; assumes that the path it
/// holds will not be changed unexpectedly by an external source.
#[derive(Clone, Debug, Default)]
pub struct Dataset {
	root: PathBuf,
	is_empty: bool,
	subset: Option<Subset>,
}

impl Dataset {
	fn new(path: PathBuf, is_empty: bool) -> Self {
		Self {
			root: path,
			is_empty,
			subset: None,
		}
	}

	pub fn add_subset(&mut self, subset: Subset, label: &mut String) {
		let temp = subset.get_full_path(self.root.clone());
		self.subset = match is_path_available(&temp) {
			Ok(()) => {
				label.replace_with(&format!("Subset: {subset}"));
				Some(subset)
			},
			Err(why) => {
				label.replace_with(&format!("Failed to select subset: {why}"));
				None
			},
		}
	}

	pub fn try_create_path(&mut self) -> io::Result<PathBuf> {
		let Some(subset) = self.subset.take() else {
			return Err(io::Error::other("subset has not been created"));
		};
		let output_folder = subset.get_full_path(self.root.clone());
		fs::create_dir_all(&output_folder)?;
		Ok(output_folder)
	}

	pub fn is_empty(&self) -> bool {
		self.is_empty
	}

	pub fn is_subset(&self) -> bool {
		self.subset.is_some()
	}

	pub fn get_root(&self) -> &Path {
		&self.root
	}
}

impl Display for Dataset {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}/", self.root.display())
	}
}

impl TryFrom<PathBuf> for Dataset {
	type Error = std::io::Error;

	/// Try to create a new dataset object from a provided [`PathBuf`].
	///
	/// The path must be a directory.
	fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
		match is_path_available(&path) {
			Ok(()) => Ok(Self::new(path, true)),
			// The root path is allow to contain files or subdirectories.
			Err(why) if why.kind() == io::ErrorKind::DirectoryNotEmpty => {
				Ok(Self::new(path, false))
			},
			Err(why) => Err(why),
		}
	}
}

#[cfg(test)]
mod tests {
	use std::assert_matches;

	use super::*;

	#[test]
	fn available_path() {
		let path = Path::new(
			"./long/path/guaranteed/to/be/available/a/a/a/a/a/a/a/a/a/a/a/a/",
		);
		assert_matches!(is_path_available(path), Ok(()))
	}

	#[test]
	fn unavailable_path_file() {
		let path = Path::new("./examples/dataset_assistant/saving/dataset.rs");
		assert_matches!(is_path_available(path), Err(why) if why.kind() == io::ErrorKind::NotADirectory)
	}

	#[test]
	fn unavailable_path_not_empty() {
		let path = Path::new("./examples/dataset_assistant/");
		assert_matches!(is_path_available(path), Err(why) if why.kind() == io::ErrorKind::DirectoryNotEmpty)
	}

	#[test]
	fn dataset_try_new() {
		let path = Path::new("./temp/dataset");
		let ds = Dataset::try_from(path.to_path_buf());
		assert_eq!(ds.unwrap().root, path.to_path_buf());
	}

	// Other method testing would be better suited for integration tests, which
	// I'm not bothering with for this.
}
