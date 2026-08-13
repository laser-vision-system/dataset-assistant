use std::{fs::File, io::Write, path::PathBuf};

use edge::Camera;
use eframe::egui::TextBuffer;

use crate::{
	cam_controls::{ControlSweepType, Finished},
	capture::Capture,
	settings::{FILE_NAME_BASE, IMAGE_EXTENSION, META_EXTENSION},
};

const STRING_LENGTH: usize = 100;

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
struct ValueStrings {
	details: String,
	short: String,
}

impl ValueStrings {
	pub fn new() -> Self {
		Self {
			details: String::with_capacity(STRING_LENGTH),
			short: String::with_capacity(STRING_LENGTH),
		}
	}
}

#[derive(Clone, Debug, Default)]
pub struct Saving {
	path: PathBuf,
	control_sweeps: Vec<ControlSweepType>,
}

impl Saving {
	pub fn new(path: PathBuf, control_sweeps: Vec<ControlSweepType>) -> Self {
		Self {
			path,
			control_sweeps,
		}
	}

	/// Save one image, incrementing controls as needed.
	pub fn pulse(
		&mut self,
		camera: &mut Camera,
		capture: &mut Capture,
		progress: &mut String,
	) -> Finished {
		let val_strs: ValueStrings =
			self.control_sweeps
				.iter()
				.fold(ValueStrings::new(), |mut acc, sweep| {
					let ctv = sweep.to_control_type_value();
					acc.details.push_str(&format!("{}\n", ctv.to_string()));
					acc.short.push_str(&ctv.current_value().to_string());
					acc
				});

		progress.replace_with(&val_strs.details);
		self.save_once(camera, capture, val_strs);

		// Increment controls for the next pass.
		if self.increment_controls(camera) {
			self.set_to_defaults(camera);
			return true;
		}
		false
	}

	/// Set all camera controls to their defaults.
	fn set_to_defaults(&mut self, camera: &mut Camera) {
		for sweep_types in &self.control_sweeps {
			sweep_types.apply_default(camera)
		}
	}

	fn increment_controls(&mut self, camera: &mut Camera) -> Finished {
		for i in (0..self.control_sweeps.len()).rev() {
			if self.control_sweeps[i].step() {
				self.control_sweeps[i].apply_current(camera);
			} else {
				self.control_sweeps[i].apply_current(camera);
				return false;
			}
		}
		true
	}

	fn save_once(
		&mut self,
		camera: &mut Camera,
		capture: &mut Capture,
		value_strings: ValueStrings,
	) {
		// To address latency, skip 4 frames.
		camera.get_image_shrink().unwrap();
		camera.get_image_shrink().unwrap();
		camera.get_image_shrink().unwrap();
		camera.get_image_shrink().unwrap();
		let image = camera.get_image_shrink().unwrap();
		capture.store_image(image);
		let mut img_filepath = self.path.clone();
		img_filepath.push(format!(
			"{}_{}.{}",
			FILE_NAME_BASE, value_strings.short, IMAGE_EXTENSION
		));
		capture.save_to_file(&img_filepath);
		let mut meta_filepath = self.path.clone();
		meta_filepath.push(format!(
			"{}_{}.{}",
			FILE_NAME_BASE, value_strings.short, META_EXTENSION
		));
		let mut output = File::create_new(meta_filepath).unwrap();
		write!(output, "{}", value_strings.details).unwrap();
	}
}
