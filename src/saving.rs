#![expect(clippy::unwrap_used, reason = "Lazy developer")]

use std::{
	fmt::Write as _,
	fs::File,
	io::Write as _,
	ops::ControlFlow,
	path::{
		Path,
		PathBuf,
	},
	sync::{
		Arc,
		Mutex,
	},
	thread,
	time::Duration,
};

use edge_cameras::{
	camera_device::{
		CameraDevice,
		CameraLike,
	},
	controls::traits::ControlValue as _,
};
use edge_core::image::Image;
use eframe::egui::TextBuffer as _;
use image::RgbImage;

use crate::{
	capture::Capture,
	runner::{
		Runner,
		RunnerError,
		RunnerJoinHandle,
	},
	settings::{
		FILE_NAME_BASE,
		IMAGE_EXTENSION,
		META_EXTENSION,
	},
	sweep::ControlSweep,
};

const EXPECTED_CONTROL_LENGTH: usize = 19;
const CAMERA_GRACE_PERIOD: Duration = Duration::from_millis(100);

#[derive(Debug)]
struct ImageMetadata {
	/// Id and value.
	detailed: String,
	/// Just the value.
	short: String,
}

impl ImageMetadata {
	/// Pre-allocate capacity for a supplied length of [`ControlSweep`]s.
	pub fn for_length(length: usize) -> Self {
		let capacity = (length + 1) * EXPECTED_CONTROL_LENGTH;
		Self {
			detailed: String::with_capacity(capacity),
			short: String::with_capacity(capacity),
		}
	}

	pub fn append_sweep(&mut self, sweep: &ControlSweep) {
		writeln!(self.detailed, "{sweep}")
			.expect("Writing into the value strings should never fail");
		let value = sweep.to_control_type_value().get().to_string();
		write!(self.short, "_{value}")
			.expect("Writing into the value strings should never fail");
	}

	pub fn append(&mut self, label: &str, value: &str) {
		writeln!(self.detailed, "{label}: {value}")
			.expect("Writing into the value strings should never fail");
		write!(self.short, "_{value}")
			.expect("Writing into the value strings should never fail");
	}
}

#[derive(Debug, Default)]
pub struct Saving {
	handle: Option<RunnerJoinHandle>,
}

impl Saving {
	/// Execute the saving procedure.
	#[expect(
		clippy::unwrap_used,
		reason = "Unlikely to happen, and panic is desired"
	)]
	pub fn start(
		destination: PathBuf,
		mut control_sweeps: Vec<ControlSweep>,
		external_intensity: Option<f32>,
		capture: Arc<Mutex<Capture>>,
		camera: Arc<Mutex<CameraDevice>>,
		progress: Arc<Mutex<String>>,
	) -> Self {
		let run = move || {
			// Technically impossible to get any other result.
			let res = loop_all_permutations(
				&destination,
				&mut control_sweeps,
				external_intensity,
				&capture,
				&camera,
				&progress,
			)
			.break_ok();
			set_to_defaults(&mut control_sweeps, &camera);
			res.map_err(|()| {
				RunnerError::new("Loop returned with a `ControlFlow::Continue`")
			})
		};
		let handle = thread::Builder::new()
			.name("saving".into())
			.spawn(run)
			.unwrap();
		Self {
			handle: Some(handle),
		}
	}
}

impl Runner for Saving {
	fn handle(&mut self) -> &mut Option<RunnerJoinHandle> {
		&mut self.handle
	}
}

/// Iterate through every control permutation, saving the result.
///
/// This function will only ever return [`ControlFlow::Break`].
fn loop_all_permutations(
	destination: &Path,
	control_sweeps: &mut [ControlSweep],
	external_intensity: Option<f32>,
	capture: &Mutex<Capture>,
	camera: &Mutex<CameraDevice>,
	progress: &Mutex<String>,
) -> ControlFlow<()> {
	loop {
		let mut val_strs = ImageMetadata::for_length(control_sweeps.len());
		if let Some(external_intensity) = external_intensity {
			let value_string = external_intensity.to_string();
			val_strs.append("externalLaserIntensity", &value_string);
		}

		let sweeps = make_owned_collection(control_sweeps);
		update_cam_controls(sweeps, camera, &mut val_strs);

		// Give the camera enough time to apply the change.
		// Especially for the exposure time, to allow the sensor to flush.
		thread::sleep(CAMERA_GRACE_PERIOD);

		progress.lock().unwrap().replace_with(&val_strs.detailed);
		let image = get_image(&mut camera.lock().unwrap());

		let img_filepath = create_image_path(destination, &val_strs.short);
		save_image_to(&mut capture.lock().unwrap(), &image, &img_filepath);

		let meta_filepath = create_metadata_path(destination, &val_strs.short);
		save_metadata_to(&meta_filepath, &val_strs.detailed);

		// Increment controls for the next pass.
		increment_controls(control_sweeps)?;
	}
}

/// Convert a mutable reference to a collection into a collection of mutable
/// references.
fn make_owned_collection(
	control_sweeps: &mut [ControlSweep],
) -> Vec<&mut ControlSweep> {
	control_sweeps.iter_mut().collect()
}

/// Update the [`CameraDevice`] controls with the [`ControlSweep`]s.
///
/// Append the [`ControlSweep`] to [`ImageMetadata`].
fn update_cam_controls(
	control_sweeps: Vec<&mut ControlSweep>,
	camera: &Mutex<CameraDevice>,
	val_strs: &mut ImageMetadata,
) {
	let mut camera = camera.lock().unwrap();

	call_on_each(control_sweeps, |sweep| {
		sweep.apply_current(&mut camera);
		val_strs.append_sweep(sweep);
	});
}

/// Call a function on each [`ControlSweep`].
fn call_on_each<F>(control_sweeps: Vec<&mut ControlSweep>, mut func: F)
where
	F: FnMut(&mut ControlSweep),
{
	for sweep in control_sweeps {
		func(sweep)
	}
}

/// Extract a decoded image from a [`CameraDevice`].
fn get_image(camera: &mut CameraDevice) -> Image {
	// To address latency, skip 1 frame.
	camera.get_frame_buffer().unwrap();
	let buffer = camera.get_frame_buffer().unwrap();
	buffer.try_decode().unwrap()
}

fn create_image_path(destination: &Path, short: &str) -> PathBuf {
	destination.join(format!("{FILE_NAME_BASE}{short}.{IMAGE_EXTENSION}"))
}

fn save_image_to(
	capture: &mut Capture,
	image: &RgbImage,
	img_filepath: &PathBuf,
) {
	capture.store_image(image);
	capture.save_to_file(img_filepath);
}

fn create_metadata_path(destination: &Path, short: &str) -> PathBuf {
	destination.join(format!("{FILE_NAME_BASE}{short}.{META_EXTENSION}"))
}

fn save_metadata_to(meta_filepath: &PathBuf, details: &str) {
	let mut output = File::create_new(meta_filepath).unwrap();
	write!(output, "{details}").unwrap();
}

/// Step once through each sweep in reverse order, short circuiting when a
/// step reports that the sweep has not completed. When all steps report
/// that they have finished, then so too does this method.
fn increment_controls(control_sweeps: &mut [ControlSweep]) -> ControlFlow<()> {
	let should_continue = control_sweeps
		.iter_mut()
		.rev()
		.any(|sweep| sweep.step().is_continue());

	if should_continue {
		ControlFlow::Continue(())
	} else {
		ControlFlow::Break(())
	}
}

/// Set all camera controls to their defaults.
fn set_to_defaults(
	control_sweeps: &mut Vec<ControlSweep>,
	camera: &Mutex<CameraDevice>,
) {
	let mut camera = camera.lock().unwrap();
	for sweep_types in control_sweeps {
		sweep_types.apply_default(&mut camera);
	}
}
