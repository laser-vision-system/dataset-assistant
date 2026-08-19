use edge::{
	Camera,
	tcam_interface::controls::{
		traits::{ControlDescription, ControlMax, ControlMin, ControlStep},
		values::ConValueIntegerType,
	},
};
use eframe::egui::{self, TextBuffer};

use crate::{
	camera_controls::ControlManager,
	entry::{
		OnDemandTextEntryBox, StaticLabelTextEntryBox, input_grid_const, input_grid_parsed_const,
	},
	settings::{
		HEADING_FONT_SIZE, HEADING_FONT_SPACING, PREHEADING_SPACE, SUBHEADING_FONT_SIZE,
		SUBHEADING_FONT_SPACING,
	},
	subset::Subset,
};

/// The information and controls shown to the user.
#[derive(Debug, Default)]
pub struct UserInterface;

impl UserInterface {
	pub fn add_heading(ui: &mut egui::Ui) {
		ui.add_space(PREHEADING_SPACE);
		ui.label(
			egui::RichText::new("Dataset Assistant")
				.size(HEADING_FONT_SIZE)
				.extra_letter_spacing(HEADING_FONT_SPACING),
		);
		ui.separator();
	}

	pub fn add_subheading(ui: &mut egui::Ui, text: &str) {
		ui.add_space(5.0);
		ui.label(
			egui::RichText::new(text)
				.size(SUBHEADING_FONT_SIZE)
				.extra_letter_spacing(SUBHEADING_FONT_SPACING),
		);
	}

	/// First option is whether the menu is open.
	/// Second option is whether a button was clicked.
	/// Third option is the index.
	pub fn camera_selection(ui: &mut egui::Ui, state: &str) -> Option<Option<Option<u32>>> {
		ui.horizontal(|ui| {
			ui.label("Selected ID:");
			ui.label(state);

			ui.menu_button("Select camera", |ui| {
				if ui.button("None").clicked() {
					return Some(None);
				}

				// Cannot be cached because state might change.
				for info in Camera::query_for_available_cameras() {
					let index = info.index().as_index().unwrap();
					let btn_lbl = format!("{}: {}", index, info.human_name());
					if ui.button(btn_lbl).clicked() {
						ui.close();
						return Some(Some(index));
					}
				}
				None
			})
			.inner
		})
		.inner
	}

	pub fn choose_folder(ui: &mut egui::Ui) -> bool {
		ui.button("Choose folder").clicked()
	}

	pub fn capture_controls(ui: &mut egui::Ui, live_feed: &mut bool) -> bool {
		ui.horizontal(|ui| {
			let btn_result = ui.add_enabled(!*live_feed, egui::Button::new("Preview capture"));
			ui.checkbox(live_feed, "Live feed");
			btn_result
		})
		.inner
		.clicked()
	}

	pub fn display_subset_entry_grid(
		ui: &mut egui::Ui,
		camera_position_input: &mut String,
		laser_intensity_input: &mut String,
	) -> Option<Subset> {
		let entries = [
			StaticLabelTextEntryBox::new("Position:", [], camera_position_input),
			StaticLabelTextEntryBox::new("Intensity:", [], laser_intensity_input),
		];
		let res = input_grid_const("subset", entries, ui);
		let trigger = res.iter().any(|r| r.lost_focus());
		let none_empty = !camera_position_input.is_empty() && !laser_intensity_input.is_empty();
		if trigger && none_empty {
			let subset = Subset::new(
				camera_position_input.to_string(),
				laser_intensity_input.to_string(),
			);
			camera_position_input.replace_with("");
			laser_intensity_input.replace_with("");
			Some(subset)
		} else {
			None
		}
	}

	pub fn draw_for_control(
		ui: &mut egui::Ui,
		manager: &mut ControlManager,
	) -> Option<ConValueIntegerType> {
		let valstr = manager.get_value().to_string();
		let minstr = manager.get_min().to_string();
		let maxstr = manager.get_max().to_string();
		let stepstr = manager.get_step().to_string();
		let entries = [
			OnDemandTextEntryBox::new(
				String::from("Change value:"),
				[valstr],
				&mut manager.value_input,
			),
			// Default
			OnDemandTextEntryBox::new(String::from("Set min:"), [minstr], &mut manager.min_input),
			OnDemandTextEntryBox::new(String::from("Set max:"), [maxstr], &mut manager.max_input),
			OnDemandTextEntryBox::new(
				String::from("Set step:"),
				[stepstr],
				&mut manager.step_input,
			),
		];

		let grid_label = manager.base.get_id().to_string();
		let res = input_grid_parsed_const(&grid_label, entries, ui);

		if let Some(new_min) = res[1] {
			manager.min = new_min;
		}

		if let Some(new_max) = res[2] {
			manager.max = new_max;
		}

		if let Some(new_step) = res[3] {
			manager.step = new_step;
		}

		match res[0] {
			Some(new_val) => {
				let _ = manager.set_value(new_val);
				Some(new_val)
			},
			None => None,
		}
	}

	pub fn draw_controls(
		ui: &mut egui::Ui,
		controls: &mut Vec<&mut ControlManager>,
	) -> Option<Vec<Option<ConValueIntegerType>>> {
		let res: Vec<_> = controls
			.into_iter()
			.map(|control_ref| {
				let name = control_ref.base.get_id().to_string();
				ui.collapsing(name, |ui| UserInterface::draw_for_control(ui, control_ref))
					.body_returned
					.flatten()
			})
			.collect();

		if res.iter().any(|val| val.is_some()) {
			Some(res)
		} else {
			None
		}
	}

	pub fn save_button(ui: &mut egui::Ui, enabled: bool) -> bool {
		let save_btn = egui::Button::new("Run save procedure");
		let response = ui.add_enabled(enabled, save_btn);
		response.clicked()
	}

	pub fn draw_collecting(ui: &mut egui::Ui, progress: &str) {
		ui.label(progress);
	}
}
