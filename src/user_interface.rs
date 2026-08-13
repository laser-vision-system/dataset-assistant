use eframe::egui;

use crate::{
	entry::{ParseAs, SimpleEntryBox, StatedEntryBox, input_grid},
	settings::{HEADING_FONT_SIZE, HEADING_FONT_SPACING, PREHEADING_SPACE},
	subset::Subset,
};

/// The information and controls shown to the user.
#[derive(Clone, Debug, Default)]
pub struct UserInterface {
	camera_input: String,
	brightness_input: String,
	camera_position_input: String,
	laser_intensity_input: String,
}

impl UserInterface {
	pub fn add_heading(&self, ui: &mut egui::Ui) {
		ui.add_space(PREHEADING_SPACE);
		ui.label(
			egui::RichText::new("Dataset Assistant")
				.size(HEADING_FONT_SIZE)
				.extra_letter_spacing(HEADING_FONT_SPACING),
		);
		ui.separator();
	}

	pub fn show_current_state(&self, ui: &mut egui::Ui, state_label: &str) {
		ui.label(state_label);
	}

	pub fn camera_entry_box(&mut self, ui: &mut egui::Ui, state: &str) -> Option<u32> {
		//ui.menu_button("menu test", |ui| {
		//	if ui.button("close").clicked() {
		//		ui.close();
		//	}
		//});

		StatedEntryBox::new("Camera id:", state, &mut self.camera_input).parse_as(ui)
	}

	pub fn choose_folder(&self, ui: &mut egui::Ui) -> bool {
		ui.button("Choose folder").clicked()
	}

	pub fn capture_controls(&mut self, ui: &mut egui::Ui, live_feed: &mut bool) -> bool {
		ui.horizontal(|ui| {
			let btn_result = ui.add_enabled(!*live_feed, egui::Button::new("Preview capture"));
			ui.checkbox(live_feed, "Live feed");
			btn_result
		})
		.inner
		.clicked()
	}

	pub fn display_subset_entry_grid(&mut self, ui: &mut egui::Ui) -> Option<Subset> {
		let entries = [
			SimpleEntryBox::new("Position:", &mut self.camera_position_input),
			SimpleEntryBox::new("Intensity:", &mut self.laser_intensity_input),
		];
		let res = input_grid("subset", entries, ui);
		let trigger = res.iter().any(|r| r.lost_focus());
		let none_empty =
			!self.camera_position_input.is_empty() && !self.laser_intensity_input.is_empty();
		if trigger && none_empty {
			let subset = Subset::new(
				self.camera_position_input.clone(),
				self.laser_intensity_input.clone(),
			);
			self.camera_position_input = String::new();
			self.laser_intensity_input = String::new();
			Some(subset)
		} else {
			None
		}
	}

	pub fn brightness_entry(&mut self, ui: &mut egui::Ui) -> Option<i64> {
		SimpleEntryBox::new("Brightness:", &mut self.brightness_input).parse_as(ui)
	}

	pub fn save_button(&mut self, ui: &mut egui::Ui, enabled: bool) -> bool {
		let save_btn = egui::Button::new("Run save procedure");
		let response = ui.add_enabled(enabled, save_btn);
		response.clicked()
	}

	pub fn draw_collecting(&mut self, ui: &mut egui::Ui, progress: &str) {
		ui.label(progress);
	}
}
