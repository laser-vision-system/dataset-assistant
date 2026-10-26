use std::{
	convert::identity,
	sync::{
		Arc,
		Mutex,
	},
};

use eframe::egui;

use crate::{
	audio::Audio,
	capture::Capture,
	state::State,
	user_interface::{
		self,
		UIComponents,
	},
};

/// The central egui application.
pub struct DatasetAssistantApp {
	state: State,
	capture: Arc<Mutex<Capture>>,
	ui: UIComponents,
	/// For playing a tone when saving completes.
	sound: Audio,
}

impl DatasetAssistantApp {
	// TODO: Do more stuff in the constructor.
	pub fn new(ctx: &egui::Context) -> Self {
		Self {
			state: State::new(),
			capture: Arc::new(Mutex::new(Capture::new(ctx))),
			ui: UIComponents::default(),
			sound: Audio::default(),
		}
	}

	fn show_controls(&mut self, ui: &mut egui::Ui) -> bool {
		user_interface::add_heading(ui);

		ui.spacing_mut().indent_ends_with_horizontal_line = true;

		ui.add_enabled_ui(self.state.current_state_is_preparation(), |ui| {
			self.directory_section(ui);
		});

		let requested_capture = self.camera_section(ui);

		self.save_section(ui);

		requested_capture
	}

	fn directory_section(&mut self, ui: &mut egui::Ui) {
		user_interface::add_subsection(ui, "Directory", |ui| {
			ui.add_enabled_ui(!self.ui.live_feed(), |ui| {
				ui.label(self.state.get_dataset_path_label());
				if user_interface::choose_folder(ui) {
					self.ui.open_folder_dialog();
				}

				ui.separator();

				ui.add_enabled_ui(self.state.dataset_not_empty(), |ui| {
					if let Some(selection) =
						user_interface::build_csv_selection(ui)
					{
						self.state.set_should_build_csv(selection);
					}
				});

				ui.add_enabled_ui(self.state.dataset_ready(), |ui| {
					ui.label(self.state.get_subset_path_label());
					if let Some(subset) = self.ui.display_subset_entry_grid(ui)
					{
						self.state.try_add_subset(subset);
					}
				});
			});
		});
	}

	fn camera_section(&mut self, ui: &mut egui::Ui) -> bool {
		user_interface::add_subsection(ui, "Camera", |ui| {
			ui.add_enabled_ui(
				!self.ui.live_feed()
					&& self.state.current_state_is_preparation(),
				|ui| {
					if let Some(selection) = user_interface::camera_selection(
						ui,
						self.state.get_camera_id_label(),
					) {
						self.state.set_requested_cam_index(selection);
					}
				},
			);

			ui.toggle_value(self.ui.show_display_mut(), "Show display panel");

			ui.separator();

			let requested_capture = ui
				.add_enabled_ui(self.state.camera_ready(), |ui| {
					ui.horizontal(|ui| {
						let btn_result = ui.add_enabled(
							!self.ui.live_feed()
								&& self.state.current_state_is_preparation(),
							egui::Button::new("Preview capture"),
						);
						ui.checkbox(self.ui.live_feed_mut(), "Live feed");
						btn_result
					})
					.inner
					.clicked()
				})
				.inner;

			if self.state.camera_ready()
				&& self.state.current_state_is_preparation()
			{
				let mut controls = self.state.get_controls();
				if controls.is_empty() {
					ui.label("No controls...");
				} else {
					ui.spacing_mut().indent_ends_with_horizontal_line = false;
					let requested_controls =
						egui::CollapsingHeader::new("Controls:")
							.default_open(false)
							.show(ui, |ui| {
								self.ui.draw_controls(ui, &mut controls)
							})
							.body_returned;
					self.state
						.set_requested_controls(requested_controls.flatten());
				}
			}

			requested_capture
		})
		.body_returned
		.is_some_and(identity)
	}

	fn save_section(&mut self, ui: &mut egui::Ui) {
		egui::Frame::default().inner_margin(8.0).show(ui, |ui| {
			ui.add_enabled_ui(
				self.state.current_state_is_preparation(),
				|ui| {
					// TODO: Move this logic out of this section.
					let res = self.ui.query_external_laser(ui);
					let (permutations, estimate) = self.state.saving_estimate();

					user_interface::draw_save_estimate(
						ui,
						permutations,
						estimate,
					);

					if user_interface::save_button(ui, self.state.is_complete())
					{
						_ = self
							.state
							.try_to_collecting(self.capture.clone(), res)
							.inspect_err(|why| {
								eprintln!("Failed to start collecting: {why}");
							});
					}
				},
			);

			ui.add_enabled_ui(self.state.current_state_is_collecting(), |ui| {
				user_interface::draw_collecting(
					ui,
					&self.state.get_saving_progress(),
				);
			});
		});
	}

	fn update(&mut self, ui: &mut egui::Ui, requested_capture: bool) {
		self.ui.update_folder_dialog(ui);
		if let Some(path) = self.ui.take_selected_folder() {
			self.state.select_dataset_folder(path);
		}
		let mut capture = self
			.capture
			.lock()
			.expect("Capture should never be poisoned");
		self.state.execute_for_preparation(
			&mut capture,
			requested_capture || self.ui.live_feed(),
		);

		self.state.execute_for_collecting(ui, &self.sound);

		self.state.execute_for_building_csv();

		if self.ui.live_feed()
			|| requested_capture
			|| self.state.current_state_is_collecting()
		{
			ui.request_repaint();
		}
	}
}

impl eframe::App for DatasetAssistantApp {
	fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
		let show_display = self.ui.show_display();

		let requested_capture = if show_display {
			let requested_capture =
				user_interface::control_panel_expanded(ui, |ui| {
					self.show_controls(ui)
				})
				.inner;

			user_interface::display_panel(ui, |ui| {
				self.capture
					.lock()
					.expect("Capture should never be poisoned")
					.display_image(ui);
			});

			requested_capture
		} else {
			user_interface::control_panel_normal(ui, |ui| {
				self.show_controls(ui)
			})
			.inner
		};

		self.update(ui, requested_capture);

		let new_show_display = self.ui.show_display();
		if show_display != new_show_display {
			if new_show_display {
				user_interface::adjust_viewport_expanded(ui);
			} else {
				user_interface::adjust_viewport_condensed(ui);
			}
		}
	}
}
