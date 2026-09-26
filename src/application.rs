use eframe::egui;

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

	fn show_controls(&mut self, ui: &mut egui::Ui) -> bool {
		UserInterface::add_heading(ui);

		self.ui_section(ui);
		ui.separator();

		let requested_capture = ui
			.add_enabled_ui(self.state.current_state_is_preparation(), |ui| {
				self.directory_section(ui);

				ui.separator();

				let requested_capture = self.camera_section(ui);

				ui.add_space(5.0);

				if UserInterface::save_button(ui, self.state.is_complete()) {
					self.state.try_to_collecting();
				}

				requested_capture
			})
			.inner;

		ui.add_enabled_ui(self.state.current_state_is_collecting(), |ui| {
			UserInterface::draw_collecting(ui, &self.state.saving_progress);
		});

		requested_capture
	}

	fn ui_section(&mut self, ui: &mut egui::Ui) {
		UserInterface::add_subheading(ui, "UI");
		ui.indent("ui", |ui| {
			ui.toggle_value(&mut self.state.show_display, "Show display panel");
		});
	}

	fn directory_section(&mut self, ui: &mut egui::Ui) {
		UserInterface::add_subheading(ui, "Directory");
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
	}

	fn camera_section(&mut self, ui: &mut egui::Ui) -> bool {
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

			let requested_capture = ui
				.add_enabled_ui(self.state.is_camera(), |ui| {
					UserInterface::capture_controls(
						ui,
						&mut self.state.live_feed,
					)
				})
				.inner;

			if self.state.is_camera() {
				ui.collapsing("Controls:", |ui| {
					self.state.requested_controls =
						UserInterface::draw_controls(
							ui,
							&mut self.state.controls.iter_mut().collect(),
						);
				});
			}

			requested_capture
		})
		.inner
	}

	fn update(&mut self, ui: &mut egui::Ui, requested_capture: bool) {
		self.state.execute_for_preparation(
			&mut self.capture,
			ui,
			requested_capture,
		);

		self.state.execute_for_collecting(&mut self.capture);

		self.state.execute_for_building_csv();

		if self.state.live_feed
			|| requested_capture
			|| self.state.current_state_is_collecting()
		{
			ui.request_repaint();
		}
	}
}

impl eframe::App for DatasetAssistantApp {
	fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
		let show_display_prior = self.state.show_display;

		if self.state.show_display {
			display_panel(ui).show(ui, |ui| {
				self.capture.display_image(ui);
			});
		}

		let requested_capture =
			control_panel(ui, |ui| self.show_controls(ui)).inner;

		self.update(ui, requested_capture);

		if show_display_prior != self.state.show_display {
			if self.state.show_display {
				when_display(ui);
			} else {
				when_no_display(ui);
			}
		}
	}
}

fn display_panel(ui: &mut egui::Ui) -> egui::Panel {
	let fill = ui.style().visuals.panel_fill;
	let frame = egui::Frame::new().inner_margin(0).fill(fill);
	egui::Panel::right("display")
		.frame(frame)
		.resizable(false)
		.min_size(TEXTURE_WIDTH as f32)
}

fn control_panel<F, R>(
	ui: &mut egui::Ui,
	add_contents: F,
) -> egui::InnerResponse<R>
where
	F: FnOnce(&mut egui::Ui) -> R,
{
	egui::CentralPanel::default_margins().show(ui, |ui| {
		ui.style_mut().spacing.scroll = egui::style::ScrollStyle::solid();

		egui::ScrollArea::vertical().show(ui, add_contents).inner
	})
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
	min: impl Into<egui::Vec2>,
	max: impl Into<egui::Vec2>,
) {
	ui.send_viewport_cmd(egui::ViewportCommand::MinInnerSize(min.into()));
	ui.send_viewport_cmd(egui::ViewportCommand::MaxInnerSize(max.into()));
}

fn set_viewport_size(ui: &mut egui::Ui, size: impl Into<egui::Vec2>) {
	ui.send_viewport_cmd(egui::ViewportCommand::InnerSize(size.into()));
}
