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

#[cfg(test)]
#[path = "dataset_tests.rs"]
mod tests;

use std::{
	fmt::Display,
	fs, io,
	path::{Path, PathBuf},
};

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

	/// Whether the root directory has anything inside it.
	is_empty: bool,

	/// A unique subset of the dataset.
	/// Is validated when set.
	subset: Option<Subset>,
}

impl Dataset {
	fn new(root: PathBuf, is_empty: bool) -> Self {
		Self {
			root,
			is_empty,
			..Default::default()
		}
	}

	pub fn add_subset(&mut self, subset: Subset) -> std::io::Result<()> {
		let temp = subset.build_full_path_on(self.root.clone());
		match is_path_available(&temp) {
			Ok(()) => {
				self.subset = Some(subset);
				Ok(())
			},
			Err(why) => {
				self.subset = None;
				Err(why)
			},
		}
	}

	pub fn try_create_path(&mut self) -> io::Result<PathBuf> {
		let Some(subset) = self.subset.take() else {
			return Err(io::Error::other("Subset has not been created"));
		};
		let output_folder = subset.build_full_path_on(self.root.clone());
		fs::create_dir_all(&output_folder)?;
		self.is_empty = false;
		Ok(output_folder)
	}

	pub fn is_empty(&self) -> bool {
		self.is_empty
	}

	pub fn not_empty(&self) -> bool {
		!self.is_empty
	}

	pub fn subset_ready(&self) -> bool {
		self.subset.is_some()
	}

	pub fn get_root(&self) -> &Path {
		&self.root
	}
}

impl Display for Dataset {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.root.display())
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
