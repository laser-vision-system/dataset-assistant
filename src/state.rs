use edge::tcam_interface::{
	Camera,
	controls::{traits::ControlDescription, values::ConValueIntegerType},
};
use eframe::egui::{self, TextBuffer};
use egui_file_dialog::FileDialog;

use crate::{
	camera_controls::{
		ControlManager, build_control_sweeps, extract_controls, try_disable_auto_exposure,
	},
	capture::Capture,
	csv_builder::{CSVBuilder, Version},
	dataset::Dataset,
	saving::Saving,
	settings::{DEFAULT_CAMERA_LABEL, DEFAULT_PATH_LABEL, DEFAULT_SUBSET_LABEL},
	subset::Subset,
};

/// A mutually exclusive collection of states that the application can be in.
#[derive(Default)]
enum Stage {
	#[default]
	Preparation,
	Collecting(Saving),
	BuildingCSV(Box<dyn CSVBuilder>),
}

/// Internal program state.
#[derive(Default)]
pub struct State {
	stage: Stage,
	file_dialog: FileDialog,
	camera: Option<Camera>,
	dataset: Option<Dataset>,
	pub camera_id_label: String,
	pub dataset_path_label: String,
	pub subset_path_label: String,
	pub requested_cam_index: Option<Option<u32>>,
	pub live_feed: bool,
	pub chose_folder: bool,
	pub saving_progress: String,
	pub camera_position_input: String,
	pub laser_intensity_input: String,
	pub controls: Vec<ControlManager>,
	pub requested_controls: Option<Vec<Option<ConValueIntegerType>>>,
	pub should_build_csv: Option<u8>,
}

impl State {
	pub fn new() -> Self {
		Self {
			camera_id_label: String::from(DEFAULT_CAMERA_LABEL),
			dataset_path_label: String::from(DEFAULT_PATH_LABEL),
			subset_path_label: String::from(DEFAULT_SUBSET_LABEL),
			..Default::default()
		}
	}

	//pub fn get_state_label(&self) -> String {
	//	// TODO: Replace this will a proper breakdown of the current program state.
	//	format!("{}\n{}", self.dataset_path_label, self.subset_path_label)
	//}

	pub fn try_add_subset(&mut self, subset: Subset) {
		self.dataset
			.as_mut()
			.unwrap()
			.add_subset(subset, &mut self.subset_path_label);
	}

	fn add_camera(&mut self, index: u32) {
		self.camera = match Camera::new(index) {
			Ok(mut camera) => {
				camera.reset_controls_to_default();
				let skip_exposure = try_disable_auto_exposure(&mut camera).is_err();
				self.controls = extract_controls(&mut camera, skip_exposure);
				Some(camera)
			},
			Err(_) => None,
		};
	}

	pub fn execute_for_preparation(
		&mut self,
		capture: &mut Capture,
		ui: &mut egui::Ui,
		request_capture: bool,
	) {
		if !self.current_state_is_preparation() {
			return;
		}

		if let Some(dataset) = self.dataset.as_ref()
			&& let Some(version) = self.should_build_csv.take()
		{
			let root_path = dataset.get_root().to_path_buf();
			let builder = Version::try_new(version, root_path)
				.expect("`build_csv_selection` should only output 0 or 1");
			self.stage = Stage::BuildingCSV(builder.into());
			return;
		}

		// Camera

		if let Some(should) = self.requested_cam_index.take() {
			match should {
				Some(index) => {
					self.add_camera(index);
					self.camera_id_label.replace_with(&format!("{index}"));
				},
				None => {
					self.camera = None;
					self.controls = Vec::new();
					self.requested_controls.take();
					self.camera_id_label.replace_with(DEFAULT_CAMERA_LABEL);
				},
			}
		}

		if let Some(cam) = self.camera.as_mut()
			&& let Some(requested_controls) = self.requested_controls.take()
		{
			debug_assert_eq!(self.controls.len(), requested_controls.len());

			for (mut value, manager) in requested_controls.into_iter().zip(&mut self.controls) {
				let Some(val) = value.take() else {
					continue;
				};
				if manager.set_value(val).is_err() {
					continue;
				}

				let id = manager.get_id();
				let newval = manager.base.get_value();
				let reason = format!("value should be valid for {}, recieved '{}'", id, val);
				cam.set_control(id, newval).expect(&reason);
			}
		}

		// Capture

		if request_capture || self.live_feed {
			let cam = self.camera.as_mut().unwrap();
			match cam.get_image_shrink() {
				Ok(image) => capture.store_image(image),
				Err(e) => eprintln!("Failed to get image: {e}"),
			}
		}

		// File dialog

		if self.chose_folder {
			self.file_dialog.pick_directory();
		}

		self.file_dialog.update(ui);

		if let Some(path) = self.file_dialog.take_picked() {
			self.dataset = Dataset::try_new(path, &mut self.dataset_path_label);
		}
	}

	pub fn execute_for_collecting(&mut self, capture: &mut Capture) {
		if let Stage::Collecting(saving) = &mut self.stage
			&& let Some(cam) = self.camera.as_mut()
		{
			if saving.pulse(cam, capture, &mut self.saving_progress) {
				// TODO: See if I can play an alert sound.
				self.stage = Stage::Preparation;
			}
		}
	}

	pub fn execute_for_building_csv(&mut self) {
		let Stage::BuildingCSV(builder) = &mut self.stage else {
			return;
		};

		if let Some(handle) = builder.is_done() {
			handle
				.join()
				.err()
				.inspect(|e| eprintln!("csv_builder returned with error: {e}"));
			self.stage = Stage::Preparation;
		}
	}

	pub fn try_to_collecting(&mut self) {
		debug_assert!(self.is_complete());
		let path = self.dataset.as_mut().unwrap().try_create_path();
		match path {
			Ok(path) => {
				let control_sweeps = build_control_sweeps(&mut self.controls);
				let saving = Saving::new(path, control_sweeps);
				self.stage = Stage::Collecting(saving);
			},
			Err(e) => eprintln!("Error saving: {e}"),
		}
	}

	pub fn current_state_is_preparation(&self) -> bool {
		matches!(self.stage, Stage::Preparation)
	}

	pub fn current_state_is_collecting(&self) -> bool {
		matches!(self.stage, Stage::Collecting(_))
	}

	#[allow(dead_code)]
	pub fn current_state_is_building_csv(&self) -> bool {
		matches!(self.stage, Stage::BuildingCSV(_))
	}

	pub fn is_complete(&self) -> bool {
		if self.camera.is_some()
			&& let Some(dataset) = self.dataset.as_ref()
			&& dataset.is_subset()
		{
			true
		} else {
			false
		}
	}

	pub fn is_camera(&self) -> bool {
		self.camera.is_some()
	}

	pub fn is_dataset(&self) -> bool {
		self.dataset.is_some()
	}
}
