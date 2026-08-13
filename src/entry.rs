use std::str::FromStr;

use eframe::egui;

pub trait GridRow {
	const COLUMNS: usize;

	fn draw(&mut self, ui: &mut egui::Ui) -> egui::Response;
}

pub trait ParseAs: GridRow {
	fn input_mut(&mut self) -> &mut String;

	fn parse_as<O>(&mut self, ui: &mut egui::Ui) -> Option<O>
	where
		O: FromStr,
	{
		let entry_res = ui.horizontal(|ui| self.draw(ui));
		if entry_res.inner.lost_focus() {
			let input = self.input_mut();
			let value = input.parse().ok();
			input.clear();
			return value;
		}
		return None;
	}
}

#[derive(Debug)]
pub struct SimpleEntryBox<'i> {
	label: &'static str,
	input: &'i mut String,
}

impl<'i> SimpleEntryBox<'i> {
	pub fn new(label: &'static str, input: &'i mut String) -> Self {
		Self { label, input }
	}
}

impl<'i> GridRow for SimpleEntryBox<'_> {
	const COLUMNS: usize = 2;

	fn draw(&mut self, ui: &mut egui::Ui) -> egui::Response {
		// Label must be created before the text entry.
		let label_widget = ui.label(self.label);
		let text_entry = egui::TextEdit::singleline(self.input)
			.hint_text("new value")
			.desired_width(f32::INFINITY);
		ui.add(text_entry).labelled_by(label_widget.id)
	}
}

impl<'i> ParseAs for SimpleEntryBox<'i> {
	fn input_mut(&mut self) -> &mut String {
		self.input
	}
}

#[derive(Debug)]
pub struct StatedEntryBox<'s, 'i> {
	label: &'static str,
	state: &'s str,
	input: &'i mut String,
}

impl<'s, 'i> StatedEntryBox<'s, 'i> {
	pub fn new(label: &'static str, state: &'s str, input: &'i mut String) -> Self {
		Self {
			label,
			state,
			input,
		}
	}
}

impl<'s, 'i> GridRow for StatedEntryBox<'_, '_> {
	const COLUMNS: usize = 3;

	fn draw(&mut self, ui: &mut egui::Ui) -> egui::Response {
		let label_widget = ui.label(self.label);
		ui.label(self.state);
		let text_entry = egui::TextEdit::singleline(self.input)
			.hint_text("new value")
			.desired_width(f32::INFINITY);
		ui.add(text_entry).labelled_by(label_widget.id)
	}
}

impl<'s, 'i> ParseAs for StatedEntryBox<'_, '_> {
	fn input_mut(&mut self) -> &mut String {
		self.input
	}
}

pub fn input_grid<T, const N: usize>(
	grid_name: &str,
	rows: [T; N],
	ui: &mut egui::Ui,
) -> [egui::Response; N]
where
	T: GridRow,
{
	egui::Grid::new(grid_name)
		.num_columns(T::COLUMNS)
		.show(ui, |ui| {
			rows.map(|mut r| {
				let res = r.draw(ui);
				ui.end_row();
				res
			})
		})
		.inner
}
