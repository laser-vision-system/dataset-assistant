use eframe::egui;

use crate::{
	capture::Capture,
	settings::{CONTROLS_WIDTH_MAX, CONTROLS_WIDTH_MIN},
	state::State,
	user_interface::UserInterface,
};

/// The central egui application.
pub struct DatasetAssistantApp {
	state: State,
	capture: Capture,
}

impl DatasetAssistantApp {
	pub fn new(ctx: &egui::Context) -> Self {
		Self {
			state: State::new(),
			capture: Capture::new(ctx),
		}
	}
}

impl eframe::App for DatasetAssistantApp {
	fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
		egui::Panel::left("controls").show(ui, |ui| {
			ui.style_mut().spacing.scroll = egui::style::ScrollStyle::solid();
			egui::ScrollArea::vertical().show(ui, |ui| {
				let mut request_capture = false;
				ui.set_width_range(CONTROLS_WIDTH_MIN..=CONTROLS_WIDTH_MAX);
				UserInterface::add_heading(ui);
				UserInterface::add_subheading(ui, "Directory");

				ui.add_enabled_ui(self.state.current_state_is_preparation(), |ui| {
					ui.indent("directory", |ui| {
						ui.add_enabled_ui(!self.state.live_feed, |ui| {
							ui.label(&self.state.dataset_path_label);
							self.state.chose_folder = UserInterface::choose_folder(ui);

							ui.separator();

							ui.label(&self.state.subset_path_label);
							ui.add_enabled_ui(self.state.is_dataset(), |ui| {
								if let Some(selection) = UserInterface::build_csv_selection(ui) {
									self.state.should_build_csv = selection;
								}

								if let Some(subset) = UserInterface::display_subset_entry_grid(
									ui,
									&mut self.state.camera_position_input,
									&mut self.state.laser_intensity_input,
								) {
									self.state.try_add_subset(subset);
								}
							});
						});
					});

					ui.separator();

					UserInterface::add_subheading(ui, "Camera");

					ui.indent("camera", |ui| {
						ui.add_enabled_ui(!self.state.live_feed, |ui| {
							if let Some(selection) =
								UserInterface::camera_selection(ui, &self.state.camera_id_label)
							{
								self.state.requested_cam_index = selection;
							}
						});

						ui.separator();

						ui.add_enabled_ui(self.state.is_camera(), |ui| {
							request_capture =
								UserInterface::capture_controls(ui, &mut self.state.live_feed);
						});

						if self.state.is_camera() {
							ui.collapsing("Controls:", |ui| {
								self.state.requested_controls = UserInterface::draw_controls(
									ui,
									&mut self.state.controls.iter_mut().collect(),
								);
							});
						}
					});

					ui.add_space(5.0);

					if UserInterface::save_button(ui, self.state.is_complete()) {
						self.state.try_to_collecting();
					}
				});

				ui.add_enabled_ui(self.state.current_state_is_collecting(), |ui| {
					UserInterface::draw_collecting(ui, &self.state.saving_progress);
				});

				self.state
					.execute_for_preparation(&mut self.capture, ui, request_capture);

				self.state.execute_for_collecting(&mut self.capture);

				self.state.execute_for_building_csv();

				if self.state.live_feed
					|| request_capture
					|| self.state.current_state_is_collecting()
				{
					ui.request_repaint();
				}
			});
		});

		let fill = ui.style().visuals.panel_fill;
		let frame = egui::Frame::new().inner_margin(0).fill(fill);
		let panel = egui::CentralPanel::default_margins().frame(frame);
		panel.show(ui, |ui| {
			self.capture.display_image(ui);
		});
	}
}
