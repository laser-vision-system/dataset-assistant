use std::{
	fmt::Display,
	fs, io,
	path::{Path, PathBuf},
};

use eframe::egui::TextBuffer as _;

use super::subset::Subset;
use crate::settings::{
	DIRECTORY_SELECTED_LABEL, FAILED_DIRECTORY_LABEL,
	NON_EMPTY_DIRECTORY_SELECTED_LABEL,
};

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
	subset: Option<Subset>,
}

impl Dataset {
	fn new(path: PathBuf) -> Self {
		Self {
			root: path,
			subset: None,
		}
	}

	/// Try to create a new dataset object from a provided [`PathBuf`].
	///
	/// The path must be a directory.
	///
	/// Details about the selected directory will be placed into the `label`
	/// parameter.
	pub fn try_new(path: PathBuf, label: &mut String) -> Option<Self> {
		match is_path_available(&path) {
			Ok(()) => {
				label.replace_with(&format!(
					"{}: {}/",
					DIRECTORY_SELECTED_LABEL,
					path.display()
				));
				Some(Self::new(path))
			},
			// The root path is allow to contain files or subdirectories.
			Err(why) if why.kind() == io::ErrorKind::DirectoryNotEmpty => {
				label.replace_with(&format!(
					"{}: {}/",
					NON_EMPTY_DIRECTORY_SELECTED_LABEL,
					path.display()
				));
				Some(Self::new(path))
			},
			Err(why) => {
				label.replace_with(&format!("{FAILED_DIRECTORY_LABEL}: {why}"));
				None
			},
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
		let mut label = String::new();
		let ds = Dataset::try_new(path.to_path_buf(), &mut label);
		assert_eq!(ds.unwrap().root, path.to_path_buf());
	}

	// Other method testing would be better suited for integration tests, which
	// I'm not bothering with for this.
}
