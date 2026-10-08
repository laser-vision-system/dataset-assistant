use eframe::egui;

use crate::entry::{ParseAs as _, StaticLabelTextEntryBox};

#[derive(Debug, Default)]
pub struct ExternalLaser {
	input_string: String,
	value: Option<f32>,
}

impl ExternalLaser {
	pub fn get_value(&self) -> Option<f32> {
		self.value
	}

	pub fn external_intensity_entry(&mut self, ui: &mut egui::Ui) {
		let labels = if let Some(current) = self.value {
			[current.to_string()]
		} else {
			[String::from("None")]
		};
		let mut entry_box = StaticLabelTextEntryBox::new(
			"External laser intensity:",
			labels,
			&mut self.input_string,
		);

		let maybe_intensity: Option<f32> = entry_box.parse_as(ui);
		if let Some(intensity) = maybe_intensity {
			self.value = Some(intensity);
		}
	}

	pub fn clear_button(&mut self, ui: &mut egui::Ui) {
		if ui.button("Clear").clicked() {
			self.value = None;
		}
	}
}
