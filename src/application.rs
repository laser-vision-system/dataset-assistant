use eframe::egui::{self, Vec2};

use crate::{
	capture::Capture,
	settings::{
		CONTROLS_WIDTH_MAX, CONTROLS_WIDTH_MIN, TEXTURE_WIDTH, WINDOW_HEIGHT,
		WINDOW_WIDTH,
	},
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

	fn main(&mut self, ui: &mut egui::Ui) {
		let mut request_capture = false;
		UserInterface::add_heading(ui);

		ui.toggle_value(&mut self.state.show_display, "Show display");

		UserInterface::add_subheading(ui, "Directory");

		ui.add_enabled_ui(self.state.current_state_is_preparation(), |ui| {
			ui.indent("directory", |ui| {
				ui.add_enabled_ui(!self.state.live_feed, |ui| {
					ui.label(&self.state.dataset_path_label);
					self.state.chose_folder = UserInterface::choose_folder(ui);

					ui.separator();

					ui.label(&self.state.subset_path_label);
					ui.add_enabled_ui(self.state.is_dataset(), |ui| {
						if let Some(selection) =
							UserInterface::build_csv_selection(ui)
						{
							self.state.should_build_csv = selection;
						}

						if let Some(subset) =
							UserInterface::display_subset_entry_grid(
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
					if let Some(selection) = UserInterface::camera_selection(
						ui,
						&self.state.camera_id_label,
					) {
						self.state.requested_cam_index = selection;
					}
				});

				ui.separator();

				ui.add_enabled_ui(self.state.is_camera(), |ui| {
					request_capture = UserInterface::capture_controls(
						ui,
						&mut self.state.live_feed,
					);
				});

				if self.state.is_camera() {
					ui.collapsing("Controls:", |ui| {
						self.state.requested_controls =
							UserInterface::draw_controls(
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

		self.state.execute_for_preparation(
			&mut self.capture,
			ui,
			request_capture,
		);

		self.state.execute_for_collecting(&mut self.capture);

		self.state.execute_for_building_csv();

		if self.state.live_feed
			|| request_capture
			|| self.state.current_state_is_collecting()
		{
			ui.request_repaint();
		}
	}
}

impl eframe::App for DatasetAssistantApp {
	fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
		let show_display_prior = self.state.show_display;

		let fill = ui.style().visuals.panel_fill;
		let frame = egui::Frame::new().inner_margin(0).fill(fill);
		let panel = egui::Panel::right("display")
			.frame(frame)
			.resizable(false)
			.min_size(TEXTURE_WIDTH as f32);
		//.max_size(TEXTURE_WIDTH as f32);
		panel.show_collapsible(ui, &mut self.state.show_display, |ui| {
			self.capture.display_image(ui);
		});

		egui::CentralPanel::default_margins().show(ui, |ui| {
			ui.style_mut().spacing.scroll = egui::style::ScrollStyle::solid();
			egui::ScrollArea::vertical().show(ui, |ui| {
				self.main(ui);
			});
		});

		if show_display_prior != self.state.show_display {
			if self.state.show_display {
				when_display(ui);
			} else {
				when_no_display(ui);
			}
		}
	}
}

fn when_display(ui: &mut egui::Ui) {
	set_viewport_bounds(ui, [WINDOW_WIDTH, WINDOW_HEIGHT], [0.0, 0.0]);
	set_viewport_size(ui, [WINDOW_WIDTH, ui.viewport_rect().height()]);
}

fn when_no_display(ui: &mut egui::Ui) {
	set_viewport_bounds(
		ui,
		[CONTROLS_WIDTH_MIN, WINDOW_HEIGHT],
		[CONTROLS_WIDTH_MAX, 0.0],
	);
	set_viewport_size(ui, [CONTROLS_WIDTH_MAX, ui.viewport_rect().height()]);
}

fn set_viewport_bounds(
	ui: &mut egui::Ui,
	min: impl Into<Vec2>,
	max: impl Into<Vec2>,
) {
	ui.send_viewport_cmd(egui::ViewportCommand::MinInnerSize(min.into()));
	ui.send_viewport_cmd(egui::ViewportCommand::MaxInnerSize(max.into()));
}

fn set_viewport_size(ui: &mut egui::Ui, size: impl Into<Vec2>) {
	ui.send_viewport_cmd(egui::ViewportCommand::InnerSize(size.into()));
}
