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
			UserInterface::draw_collecting(
				ui,
				self.state.get_saving_progress(),
			);
		});

		requested_capture
	}

	fn ui_section(&mut self, ui: &mut egui::Ui) {
		UserInterface::add_subheading(ui, "UI");
		ui.indent("ui", |ui| {
			ui.toggle_value(
				self.state.get_show_display_mut(),
				"Show display panel",
			);
		});
	}

	fn directory_section(&mut self, ui: &mut egui::Ui) {
		UserInterface::add_subheading(ui, "Directory");
		ui.indent("directory", |ui| {
			ui.add_enabled_ui(!self.state.get_live_feed(), |ui| {
				ui.label(self.state.get_dataset_path_label());
				self.state
					.set_chose_folder(UserInterface::choose_folder(ui));

				ui.separator();

				ui.label(self.state.get_subset_path_label());
				ui.add_enabled_ui(self.state.is_dataset(), |ui| {
					if let Some(selection) =
						UserInterface::build_csv_selection(ui)
					{
						self.state.set_should_build_csv(selection);
					}

					let (camera_position_input, laser_intensity_input) =
						self.state.get_subset_inputs();
					if let Some(subset) =
						UserInterface::display_subset_entry_grid(
							ui,
							camera_position_input,
							laser_intensity_input,
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
			ui.add_enabled_ui(!self.state.get_live_feed(), |ui| {
				if let Some(selection) = UserInterface::camera_selection(
					ui,
					self.state.get_camera_id_label(),
				) {
					self.state.set_requested_cam_index(selection);
				}
			});

			ui.separator();

			let requested_capture = ui
				.add_enabled_ui(self.state.is_camera(), |ui| {
					UserInterface::capture_controls(
						ui,
						self.state.get_live_feed_mut(),
					)
				})
				.inner;

			if self.state.is_camera() {
				ui.collapsing("Controls:", |ui| {
					let mut controls = self.state.get_controls();
					let requested_controls =
						UserInterface::draw_controls(ui, &mut controls);
					self.state.set_requested_controls(requested_controls);
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

		if show_display {
			display_panel(ui).show(ui, |ui| {
				self.capture.display_image(ui);
			});
		}

		let requested_capture =
			control_panel(ui, |ui| self.show_controls(ui)).inner;

		self.update(ui, requested_capture);

		let new_show_display = self.state.get_show_display();
		if show_display != new_show_display {
			if new_show_display {
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
