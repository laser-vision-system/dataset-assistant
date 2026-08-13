use eframe::egui;

use crate::{
	capture::Capture, settings::CONTROLS_WIDTH, state::State, user_interface::UserInterface,
};

/// The central egui application.
pub struct DatasetAssistantApp {
	state: State,
	user_interface: UserInterface,
	capture: Capture,
}

impl DatasetAssistantApp {
	pub fn new(ctx: &egui::Context) -> Self {
		Self {
			state: State::new(),
			user_interface: Default::default(),
			capture: Capture::new(ctx),
		}
	}
}

impl eframe::App for DatasetAssistantApp {
	fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
		egui::Panel::left("controls").show(ui, |ui| {
			ui.set_min_width(CONTROLS_WIDTH);
			self.user_interface.add_heading(ui);

			let mut request_capture = false;

			ui.add_enabled_ui(self.state.current_state_is_preparation(), |ui| {
				ui.add_enabled_ui(!self.state.live_feed, |ui| {
					self.state.chose_folder = self.user_interface.choose_folder(ui);
					ui.add_enabled_ui(self.state.is_dataset(), |ui| {
						if let Some(subset) = self.user_interface.display_subset_entry_grid(ui) {
							self.state.try_add_subset(subset);
						}
					});
				});
				self.user_interface
					.show_current_state(ui, &self.state.get_state_label());

				ui.separator();

				ui.add_enabled_ui(!self.state.live_feed, |ui| {
					self.state.requested_cam_index = self
						.user_interface
						.camera_entry_box(ui, &self.state.camera_id_label);
				});
				ui.add_enabled_ui(self.state.is_camera(), |ui| {
					request_capture = self
						.user_interface
						.capture_controls(ui, &mut self.state.live_feed);
					ui.add_enabled_ui(!self.state.live_feed, |ui| {
						self.state.requested_brightness = self.user_interface.brightness_entry(ui);
					});
				});

				ui.separator();

				if self
					.user_interface
					.save_button(ui, self.state.is_complete())
				{
					self.state.try_to_collecting();
				}
			});

			ui.add_enabled_ui(self.state.current_state_is_collecting(), |ui| {
				self.user_interface
					.draw_collecting(ui, &self.state.saving_progress);
			});

			self.state
				.execute_for_preparation(&mut self.capture, ui, request_capture);

			self.state.execute_for_collecting(&mut self.capture);

			if self.state.live_feed || request_capture || self.state.current_state_is_collecting() {
				ui.request_repaint();
			}
		});

		let fill = ui.style().visuals.panel_fill;
		let frame = egui::Frame::new().inner_margin(0).fill(fill);
		let panel = egui::CentralPanel::default_margins().frame(frame);
		panel.show(ui, |ui| {
			self.capture.display_image(ui);
		});
	}
}
