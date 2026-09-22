use std::{fs::File, io::Write, path::PathBuf};

use edge::tcam_interface::{CameraDevice, controls::traits::ControlValue};
use eframe::egui::TextBuffer;

use crate::{
	capture::Capture,
	settings::{FILE_NAME_BASE, IMAGE_EXTENSION, META_EXTENSION},
	sweep::{ControlSweep, Finished},
};

const EXPECTED_CONTROL_LENGTH: usize = 19;

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
struct ValueStrings {
	/// Id and value.
	details: String,
	/// Just the value.
	short: String,
}

impl ValueStrings {
	pub fn with_capacity(capacity: usize) -> Self {
		Self {
			details: String::with_capacity(capacity * EXPECTED_CONTROL_LENGTH),
			short: String::with_capacity(capacity * EXPECTED_CONTROL_LENGTH),
		}
	}
}

#[derive(Debug, Default)]
pub struct Saving {
	path: PathBuf,
	control_sweeps: Vec<ControlSweep>,
}

impl Saving {
	pub fn new(path: PathBuf, control_sweeps: Vec<ControlSweep>) -> Self {
		Self {
			path,
			control_sweeps,
		}
	}

	/// Save one image, incrementing controls as needed.
	pub fn pulse(
		&mut self,
		camera: &mut CameraDevice,
		capture: &mut Capture,
		progress: &mut String,
	) -> Finished {
		let mut val_strs = ValueStrings::with_capacity(self.control_sweeps.len());
		for sweep in &mut self.control_sweeps {
			sweep.apply_current(camera);
			val_strs
				.details
				.push_str(&format!("{}\n", sweep.to_string()));
			let value = sweep.to_control_type_value().get_value().to_string();
			val_strs.short.push_str(&format!("_{}", value));
		}

		progress.replace_with(&val_strs.details);
		self.save_once(camera, capture, val_strs);

		// Increment controls for the next pass.
		if self.increment_controls() {
			self.set_to_defaults(camera);
			return true;
		}
		false
	}

	/// Set all camera controls to their defaults.
	fn set_to_defaults(&mut self, camera: &mut CameraDevice) {
		for sweep_types in &mut self.control_sweeps {
			sweep_types.apply_default(camera)
		}
	}

	/// Step once through each sweep in reverse order, short circuiting when a step reports that the sweep has not completed.
	/// When all steps report that they have finished, then so too does this method.
	fn increment_controls(&mut self) -> Finished {
		self.control_sweeps.iter_mut().rev().all(|c| c.step())
	}

	fn save_once(
		&mut self,
		camera: &mut CameraDevice,
		capture: &mut Capture,
		value_strings: ValueStrings,
	) {
		// To address latency, skip 4 frames.
		camera.get_frame().unwrap();
		camera.get_frame().unwrap();
		camera.get_frame().unwrap();
		camera.get_frame().unwrap();
		let frame = camera.get_frame().unwrap();
		let image = camera.decode_image(frame).unwrap();
		capture.store_image(image);
		let mut img_filepath = self.path.clone();
		img_filepath.push(format!(
			"{}{}.{}",
			FILE_NAME_BASE, value_strings.short, IMAGE_EXTENSION
		));
		capture.save_to_file(&img_filepath);
		let mut meta_filepath = self.path.clone();
		meta_filepath.push(format!(
			"{}{}.{}",
			FILE_NAME_BASE, value_strings.short, META_EXTENSION
		));
		let mut output = File::create_new(meta_filepath).unwrap();
		write!(output, "{}", value_strings.details).unwrap();
	}
}
