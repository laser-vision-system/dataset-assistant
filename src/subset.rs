//! A subset within a dataset is a combination of the position and intensity.
//!
//! Necessary because you cannot set the laser intensity path without the
//! camera position path.
//!
//! Each subset is unique, so a new one cannot be created with the same values
//! as one that already exists.

use std::{fmt::Display, path::PathBuf};

#[derive(Clone, Debug, Default)]
pub struct Subset {
	camera_position: String,
	laser_intensity: String,
}

impl Subset {
	pub fn new(camera_position: String, laser_intensity: String) -> Self {
		Self {
			camera_position,
			laser_intensity,
		}
	}

	pub fn camera_position(&self) -> &str {
		&self.camera_position
	}

	pub fn laser_intensity(&self) -> &str {
		&self.laser_intensity
	}

	pub fn build_full_path_on(&self, mut root: PathBuf) -> PathBuf {
		root.push(self.camera_position());
		root.push(self.laser_intensity());
		root
	}
}

impl Display for Subset {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}/{}", self.camera_position, self.laser_intensity)
	}
}
