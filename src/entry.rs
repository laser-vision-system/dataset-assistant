use std::str::FromStr;

use eframe::egui;

/// An entry is a collection of elements used for user entry.
///
/// Only a single output is supported.
pub trait Entry {
	const ELEMENTS: usize;

	fn draw(&mut self, ui: &mut egui::Ui) -> egui::Response;
}

/// Display an [`EntryRow`] immediately and get a response.
///
/// Expects a single input source.
pub trait ParseAs: Entry {
	fn input_mut(&mut self) -> &mut String;

	fn parse_as<O>(&mut self, ui: &mut egui::Ui) -> Option<O>
	where
		O: FromStr,
	{
		let entry_res = self.draw(ui);
		if entry_res.lost_focus() {
			let input = self.input_mut();
			let value = input.parse().ok();
			input.clear();
			return value;
		}
		return None;
	}
}

/// A label and single-line text entry box.
///
/// Needs to use a reference because it is consumed when displayed.
#[derive(Debug)]
pub struct StaticLabelTextEntryBox<'i> {
	label: &'static str,
	input: &'i mut String,
}

impl<'i> StaticLabelTextEntryBox<'i> {
	pub fn new(label: &'static str, input: &'i mut String) -> Self {
		Self { label, input }
	}
}

impl<'i> Entry for StaticLabelTextEntryBox<'_> {
	const ELEMENTS: usize = 2;

	fn draw(&mut self, ui: &mut egui::Ui) -> egui::Response {
		// Label must be created before the text entry.
		let label_widget = ui.label(self.label);
		let text_entry = egui::TextEdit::singleline(self.input)
			.hint_text("new value")
			.desired_width(f32::INFINITY);
		ui.add(text_entry).labelled_by(label_widget.id)
	}
}

impl<'i> ParseAs for StaticLabelTextEntryBox<'_> {
	fn input_mut(&mut self) -> &mut String {
		self.input
	}
}

/// A label and single-line text entry box.
///
/// Needs to use a reference because it is consumed when displayed.
#[derive(Debug)]
pub struct OnDemandTextEntryBox<'i, const N: usize> {
	label: String,
	state: [String; N],
	input: &'i mut String,
}

impl<'i, const N: usize> OnDemandTextEntryBox<'i, N> {
	pub fn new(label: String, state: [String; N], input: &'i mut String) -> Self {
		Self {
			label,
			state,
			input,
		}
	}
}

impl<'i, const N: usize> Entry for OnDemandTextEntryBox<'_, N> {
	const ELEMENTS: usize = 2 + N;

	fn draw(&mut self, ui: &mut egui::Ui) -> egui::Response {
		// Label must be created before the text entry.
		let label_widget = ui.label(&self.label);
		for lbl in &self.state {
			ui.label(lbl);
		}
		let text_entry = egui::TextEdit::singleline(self.input)
			.hint_text("new value")
			.desired_width(f32::INFINITY);
		ui.add(text_entry).labelled_by(label_widget.id)
	}
}

impl<'i, const N: usize> ParseAs for OnDemandTextEntryBox<'_, N> {
	fn input_mut(&mut self) -> &mut String {
		self.input
	}
}

pub fn input_grid_fixed<T, const N: usize>(
	grid_name: &str,
	rows: [T; N],
	ui: &mut egui::Ui,
) -> [egui::Response; N]
where
	T: Entry,
{
	egui::Grid::new(grid_name)
		.num_columns(T::ELEMENTS)
		.show(ui, |ui| {
			rows.map(|mut r| {
				let res = r.draw(ui);
				ui.end_row();
				res
			})
		})
		.inner
}

pub fn input_grid_dyn<T, R, O>(
	grid_name: &str,
	rows: Vec<T>,
	ui: &mut egui::Ui,
) -> egui::InnerResponse<R>
where
	T: ParseAs,
	R: FromIterator<Option<O>>,
	O: FromStr,
{
	egui::Grid::new(grid_name)
		.num_columns(T::ELEMENTS)
		.show(ui, |ui| {
			rows.into_iter()
				.map(|mut r| {
					let res = r.parse_as::<O>(ui);
					ui.end_row();
					res
				})
				.collect()
		})
}
