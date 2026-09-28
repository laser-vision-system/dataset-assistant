use eframe::egui;

use crate::{
	capture::Capture,
	state::State,
	user_interface::{self, UIComponents},
};

/// The central egui application.
pub struct DatasetAssistantApp {
	state: State,
	capture: Capture,
	ui: UIComponents,
}

impl DatasetAssistantApp {
	// TODO: Do more stuff in the constructor.
	pub fn new(ctx: &egui::Context) -> Self {
		Self {
			state: State::new(),
			capture: Capture::new(ctx),
			ui: UIComponents::default(),
		}
	}

	fn show_controls(&mut self, ui: &mut egui::Ui) -> bool {
		user_interface::add_heading(ui);

		ui.spacing_mut().indent_ends_with_horizontal_line = true;

		let requested_capture = ui
			.add_enabled_ui(self.state.current_state_is_preparation(), |ui| {
				self.directory_section(ui);
				self.camera_section(ui)
			})
			.inner;

		self.save_section(ui);

		requested_capture
	}

	fn directory_section(&mut self, ui: &mut egui::Ui) {
		user_interface::add_subsection(ui, "Directory", |ui| {
			ui.add_enabled_ui(!self.state.get_live_feed(), |ui| {
				ui.label(self.state.get_dataset_path_label());
				self.state
					.set_chose_folder(user_interface::choose_folder(ui));

				ui.separator();

				ui.add_enabled_ui(self.state.dataset_ready(), |ui| {
					ui.label(self.state.get_subset_path_label());
					let (camera_position_input, laser_intensity_input) =
						self.state.get_subset_inputs();
					if let Some(subset) =
						user_interface::display_subset_entry_grid(
							ui,
							camera_position_input,
							laser_intensity_input,
						) {
						self.state.try_add_subset(subset);
					}

					ui.add_enabled_ui(self.state.subset_ready(), |ui| {
						if let Some(selection) =
							user_interface::build_csv_selection(ui)
						{
							self.state.set_should_build_csv(selection);
						}
					});
				});
			});
		});
	}

	fn camera_section(&mut self, ui: &mut egui::Ui) -> bool {
		user_interface::add_subsection(ui, "Camera", |ui| {
			ui.add_enabled_ui(!self.state.get_live_feed(), |ui| {
				if let Some(selection) = user_interface::camera_selection(
					ui,
					self.state.get_camera_id_label(),
				) {
					self.state.set_requested_cam_index(selection);
				}
			});

			ui.separator();

			let requested_capture = ui
				.add_enabled_ui(self.state.camera_ready(), |ui| {
					ui.toggle_value(
						self.state.get_show_display_mut(),
						"Show display panel",
					);
					user_interface::capture_controls(
						ui,
						self.state.get_live_feed_mut(),
					)
				})
				.inner;

			if self.state.camera_ready() {
				let mut controls = self.state.get_controls();
				if controls.is_empty() {
					ui.label("No controls...");
				} else {
					ui.spacing_mut().indent_ends_with_horizontal_line = false;
					let requested_controls =
						egui::CollapsingHeader::new("Controls:")
							.default_open(false)
							.show(ui, |ui| {
								user_interface::draw_controls(ui, &mut controls)
							})
							.body_returned;
					self.state
						.set_requested_controls(requested_controls.flatten());
				}
			}

			requested_capture
		})
		.body_returned
		.is_some_and(|requested| requested)
	}

	fn save_section(&mut self, ui: &mut egui::Ui) {
		egui::Frame::default().inner_margin(8.0).show(ui, |ui| {
			ui.add_enabled_ui(
				self.state.current_state_is_preparation(),
				|ui| {
					let res = self.ui.query_external_laser(ui);
					if user_interface::save_button(ui, self.state.is_complete())
					{
						self.state.try_to_collecting(res);
					}
				},
			);

			ui.add_enabled_ui(self.state.current_state_is_collecting(), |ui| {
				user_interface::draw_collecting(
					ui,
					self.state.get_saving_progress(),
				);
			});
		});
	}

	fn update(&mut self, ui: &mut egui::Ui, requested_capture: bool) {
		self.state.execute_for_preparation(
			&mut self.capture,
			ui,
			requested_capture,
		);

		self.state.execute_for_collecting(&mut self.capture);

		self.state.execute_for_building_csv();

		if self.state.get_live_feed()
			|| requested_capture
			|| self.state.current_state_is_collecting()
		{
			ui.request_repaint();
		}
	}
}

impl eframe::App for DatasetAssistantApp {
	fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
		let show_display = self.state.get_show_display();

		let requested_capture = if show_display {
			let requested_capture =
				user_interface::control_panel_expanded(ui, |ui| {
					self.show_controls(ui)
				})
				.inner;

			user_interface::display_panel(ui, |ui| {
				self.capture.display_image(ui);
			});

			requested_capture
		} else {
			user_interface::control_panel_normal(ui, |ui| {
				self.show_controls(ui)
			})
			.inner
		};

		self.update(ui, requested_capture);

		let new_show_display = self.state.get_show_display();
		if show_display != new_show_display {
			if new_show_display {
				user_interface::adjust_viewport_expanded(ui);
			} else {
				user_interface::adjust_viewport_condensed(ui);
			}
		}
	}
}
