#![expect(clippy::unwrap_used, reason = "Lazy developer")]
#![expect(clippy::print_stderr, reason = "Acceptable in this project")]

use edge::tcam_interface::{
	CameraDevice,
	controls::{traits::ControlDescription as _, values::ConValueIntegerType},
};
use eframe::egui::{self, TextBuffer as _};
use egui_file_dialog::FileDialog;

use crate::{
	camera_controls::{
		ControlManager, build_control_sweeps, extract_controls,
		try_disable_auto_exposure,
	},
	capture::Capture,
	csv_builder::Version,
	dataset::Dataset,
	runner::Runner,
	saving::Saving,
	settings::{
		DEFAULT_CAMERA_LABEL, DEFAULT_PATH_LABEL, DEFAULT_SUBSET_LABEL,
		DIRECTORY_SELECTED_LABEL, FAILED_DIRECTORY_LABEL,
		NON_EMPTY_DIRECTORY_SELECTED_LABEL,
	},
	subset::Subset,
};

/// A mutually exclusive collection of states that the application can be in.
#[derive(Default)]
enum Stage {
	#[default]
	Preparation,
	Collecting(Saving),
	BuildingCSV(Box<dyn Runner>),
}

/// Internal program state.
#[expect(clippy::option_option, reason = "It works okay")]
#[derive(Default)]
pub struct State {
	stage: Stage,

	/// Set externally, consumed when read; glorified parameter input.
	/// Will change [`stage`](Stage), not coupled to it.
	/// Transient.
	should_build_csv: Option<u8>,

	/// Set externally, read locally; glorified parameter input.
	/// Remotely polls button input.
	/// Coupled to [`file_dialog`](FileDialog).
	/// Semi-transient.
	chose_folder: bool,
	/// Only accessed mutably. Uses chose folder to produce a dataset.
	/// Cohesion with [`dataset`](Dataset).
	/// Persistent.
	file_dialog: FileDialog,
	/// Created by file dialog and used to set subset, try for collecting, or
	/// build csv. Only set back to None when selecting an invalid path.
	/// Deeply coupled [here](self).
	/// Semi-persistent.
	dataset: Option<Dataset>,
	/// Set when choosing dataset, only diplayed once.
	/// Coupled to [here](self) and [`dataset`](Dataset).
	/// Persistent.
	dataset_path_label: String,

	/// These two are only ever used in the UI for the subset grid.
	/// Coupled to [`Subset`].
	/// Persistent.
	camera_position_input: String,
	laser_intensity_input: String,
	/// Set when adding subset, only diplayed once.
	/// Coupled to [here](self) and [`dataset`](Dataset)/[`Subset`].
	/// Persistent.
	subset_path_label: String,

	/// Set externally, consumed when read; glorified parameter input.
	/// Coupled to [`camera`](CameraDevice).
	/// Transient.
	requested_cam_index: Option<Option<u32>>,
	/// Only set with requested cam index, used for controls, capture, and
	/// collecting. Deeply coupled [here](self).
	/// Semi-peristent.
	camera: Option<CameraDevice>,
	/// Set when selecting camera, only diplayed once.
	/// Coupled to [here](self) and [`camera`](CameraDevice).
	/// Persistent.
	camera_id_label: String,
	/// Set when adding camera, used when changing controls or building sweeps
	/// for collecting. Coupled to [`camera`](CameraDevice).
	/// Semi-persistent.
	controls: Vec<ControlManager>,
	/// Set externally, consumed when read; glorified parameter input.
	/// Coupled to [`camera`](CameraDevice) and [`controls`](ControlManager).
	/// Transient.
	requested_controls: Option<Vec<Option<ConValueIntegerType>>>,

	/// Toggled in one placed and use extensively in app and ui.
	/// Locally guards reading from camera.
	/// Lightly coupled to [`camera`](CameraDevice).
	/// Persistent.
	live_feed: bool,

	/// Set and used externally. Never used locally.
	/// Intended to be used with live feed.
	/// Peristent.
	show_display: bool,

	/// Set in saving pulse and used in UI draw collecting.
	/// Coupled to [here](self) and [`Saving`].
	/// Persistent.
	saving_progress_label: String,
}

/// Associated functions.
impl State {
	pub fn new() -> Self {
		Self {
			camera_id_label: String::from(DEFAULT_CAMERA_LABEL),
			dataset_path_label: String::from(DEFAULT_PATH_LABEL),
			subset_path_label: String::from(DEFAULT_SUBSET_LABEL),
			..Default::default()
		}
	}
}

/// Queries.
impl State {
	pub fn current_state_is_preparation(&self) -> bool {
		matches!(self.stage, Stage::Preparation)
	}

	pub fn current_state_is_collecting(&self) -> bool {
		matches!(self.stage, Stage::Collecting(_))
	}

	pub fn is_complete(&self) -> bool {
		self.camera_ready() && self.subset_ready()
	}

	pub fn camera_ready(&self) -> bool {
		self.camera.is_some()
	}

	pub fn dataset_ready(&self) -> bool {
		self.dataset.is_some()
	}

	pub fn dataset_not_empty(&self) -> bool {
		self.dataset.as_ref().is_some_and(Dataset::not_empty)
	}

	pub fn subset_ready(&self) -> bool {
		self.dataset.as_ref().is_some_and(Dataset::subset_ready)
	}
}

/// Setters.
impl State {
	#[expect(clippy::needless_pass_by_value, reason = "Value is consumed")]
	pub fn try_add_subset(&mut self, subset: Subset) {
		let Some(dataset) = self.dataset.as_mut() else {
			eprintln!("Attempted to add subset with no dataset!");
			return;
		};
		let subset_label = match dataset.add_subset(subset.clone()) {
			Ok(()) => {
				format!("Subset: {subset}")
			},
			Err(why) => {
				format!("Failed to select subset: {why}")
			},
		};
		self.subset_path_label.replace_with(&subset_label);
	}

	fn add_camera(&mut self, index: u32) {
		let index = nokhwa::utils::CameraIndex::Index(index);
		self.camera = match CameraDevice::try_new_open(index) {
			Ok(mut camera) => {
				// camera.reset_controls_to_default();
				let skip_exposure =
					try_disable_auto_exposure(&mut camera).is_err();
				self.controls = extract_controls(&mut camera, skip_exposure);
				Some(camera)
			},
			Err(_) => None,
		};
	}

	pub fn set_chose_folder(&mut self, state: bool) {
		self.chose_folder = state;
	}

	pub fn set_should_build_csv(&mut self, state: Option<u8>) {
		self.should_build_csv = state;
	}

	#[expect(clippy::option_option, reason = "It works okay")]
	pub fn set_requested_cam_index(&mut self, state: Option<Option<u32>>) {
		self.requested_cam_index = state;
	}

	pub fn set_requested_controls(
		&mut self,
		state: Option<Vec<Option<ConValueIntegerType>>>,
	) {
		self.requested_controls = state;
	}
}

/// Getters.
impl State {
	pub fn get_saving_progress(&self) -> &str {
		&self.saving_progress_label
	}

	pub fn get_show_display(&mut self) -> bool {
		self.show_display
	}

	pub fn get_show_display_mut(&mut self) -> &mut bool {
		&mut self.show_display
	}

	pub fn get_live_feed(&self) -> bool {
		self.live_feed
	}

	pub fn get_live_feed_mut(&mut self) -> &mut bool {
		&mut self.live_feed
	}

	pub fn get_dataset_path_label(&self) -> &str {
		&self.dataset_path_label
	}

	pub fn get_subset_path_label(&self) -> &str {
		&self.subset_path_label
	}

	pub fn get_subset_inputs(&mut self) -> (&mut String, &mut String) {
		(
			&mut self.camera_position_input,
			&mut self.laser_intensity_input,
		)
	}

	pub fn get_camera_id_label(&self) -> &str {
		&self.camera_id_label
	}

	pub fn get_controls(&mut self) -> Vec<&mut ControlManager> {
		self.controls.iter_mut().collect()
	}
}

/// State changers.
impl State {
	pub fn try_to_collecting(
		&mut self,
		external_laser: Option<f32>,
	) -> Result<(), String> {
		let dataset = self
			.dataset
			.as_mut()
			.ok_or_else(|| String::from("No dataset present"))?;
		let path = dataset.try_create_path().map_err(|why| {
			format!("Failed to create path for collecting: {why}")
		})?;

		let control_sweeps = build_control_sweeps(&self.controls);
		let saving = Saving::new(path, control_sweeps, external_laser);
		self.stage = Stage::Collecting(saving);

		Ok(())
	}
}

/// Preparation state.
impl State {
	pub fn execute_for_preparation(
		&mut self,
		capture: &mut Capture,
		ui: &mut egui::Ui,
		request_capture: bool,
	) {
		if !self.current_state_is_preparation() {
			return;
		}

		if self.prep_build_csv() {
			return;
		}

		self.prep_cam_dev();
		self.prep_cam_controls();
		self.prep_capture(capture, request_capture);
		self.prep_file_dialog(ui);
	}

	fn prep_build_csv(&mut self) -> bool {
		if let Some(dataset) = self.dataset.as_ref()
			&& let Some(version) = self.should_build_csv.take()
		{
			let root_path = dataset.get_root().to_path_buf();
			let builder = Version::try_new(version, root_path)
				.expect("`build_csv_selection` should only output 0 or 1");
			self.stage = Stage::BuildingCSV(builder);
			return true;
		}
		false
	}

	fn prep_cam_dev(&mut self) {
		if let Some(should) = self.requested_cam_index.take() {
			if let Some(index) = should {
				self.add_camera(index);
				self.camera_id_label.replace_with(&format!("{index}"));
			} else {
				self.camera = None;
				self.controls = Vec::new();
				self.requested_controls.take();
				self.camera_id_label.replace_with(DEFAULT_CAMERA_LABEL);
			}
		}
	}

	fn prep_cam_controls(&mut self) {
		if let Some(cam) = self.camera.as_mut()
			&& let Some(requested_controls) = self.requested_controls.take()
		{
			debug_assert_eq!(
				self.controls.len(),
				requested_controls.len(),
				"Different sizes will cause data loss"
			);

			for (mut value, manager) in
				requested_controls.into_iter().zip(&mut self.controls)
			{
				let Some(val) = value.take() else {
					continue;
				};
				if manager.set_value(val).is_err() {
					continue;
				}

				let id = manager.get_id();
				let newval = manager.base.get_value();
				let reason =
					format!("value should be valid for {id}, recieved '{val}'");
				cam.set_control(id, newval).expect(&reason);
			}
		}
	}

	fn prep_capture(&mut self, capture: &mut Capture, request_capture: bool) {
		if request_capture || self.live_feed {
			let cam = self.camera.as_mut().unwrap();
			let buffer = match cam.get_frame_buffer() {
				Ok(frame) => frame,
				Err(why) => {
					eprintln!("Failed to get frame: {why}");
					return;
				},
			};
			let image = match cam.decode_buffer(buffer) {
				Ok(image) => image,
				Err(why) => {
					eprintln!("Failed to decode to image: {why}");
					return;
				},
			};
			capture.store_image(&image);
		}
	}

	fn prep_file_dialog(&mut self, ui: &mut egui::Ui) {
		if self.chose_folder {
			self.file_dialog.pick_directory();
		}

		self.file_dialog.update(ui);

		if let Some(path) = self.file_dialog.take_picked() {
			let maybe_dataset = Dataset::try_from(path);
			match maybe_dataset {
				Ok(dataset) => {
					let new_label = if dataset.is_empty() {
						format!("{DIRECTORY_SELECTED_LABEL}: {dataset}/")
					} else {
						format!(
							"{NON_EMPTY_DIRECTORY_SELECTED_LABEL}: {dataset}/"
						)
					};
					self.dataset_path_label.replace_with(&new_label);
					self.dataset = Some(dataset);
				},
				Err(why) => {
					//
					self.dataset_path_label.replace_with(&format!(
						"{FAILED_DIRECTORY_LABEL}: {why}"
					));
				},
			}
		}
	}
}

/// `Collecting` state.
impl State {
	pub fn execute_for_collecting(&mut self, capture: &mut Capture) {
		if let Stage::Collecting(saving) = &mut self.stage
			&& let Some(cam) = self.camera.as_mut()
			&& saving
				.pulse(cam, capture, &mut self.saving_progress_label)
				.is_break()
		{
			// TODO: See if I can play an alert sound.
			self.stage = Stage::Preparation;
		}
	}
}

/// `BuildingCSV` state.
impl State {
	pub fn execute_for_building_csv(&mut self) {
		let Stage::BuildingCSV(builder) = &mut self.stage else {
			return;
		};

		if let Some(handle) = builder.is_done() {
			handle.join().err().inspect(|why| {
				eprintln!("csv_builder returned with error: {why}");
			});
			self.stage = Stage::Preparation;
		}
	}
}
