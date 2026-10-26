//! The information and controls shown to the user.

pub mod external_laser;

use std::time::Duration;

use edge_cameras::{
	camera_device::query_for_available_cameras,
	controls::{
		traits::{
			ControlDefault as _,
			ControlDescription as _,
			ControlMax as _,
			ControlMin as _,
			ControlStep as _,
		},
		values::ConValueIntegerType,
	},
};
use eframe::egui;

use self::external_laser::ExternalLaser;
use crate::{
	camera_controls::ControlManager,
	entry::{
		OnDemandTextEntryBox,
		StaticLabelTextEntryBox,
		input_grid_const,
		input_grid_parsed_const,
	},
	settings::{
		CONTROLS_MAX_WIDTH,
		CONTROLS_MIN_WIDTH,
		DISPLAY_MIN_HEIGHT,
		DISPLAY_MIN_WIDTH,
		EXPANDED_MAX_HEIGHT,
		EXPANDED_MAX_WIDTH,
		EXPANDED_MIN_HEIGHT,
		EXPANDED_MIN_WIDTH,
		FRAME_INNER_MARGIN,
		FRAME_STROKE_GREY,
		FRAME_STROKE_WIDTH,
		HEADING_FONT_SIZE,
		HEADING_FONT_SPACING,
		PREHEADING_SPACE,
		SUBHEADING_FONT_SIZE,
		SUBHEADING_FONT_SPACING,
	},
	subset::Subset,
};

#[derive(Debug, Default)]
pub struct UIComponents {
	external_laser: ExternalLaser,
}

impl UIComponents {
	pub fn query_external_laser(&mut self, ui: &mut egui::Ui) -> Option<f32> {
		ui.horizontal(|ui| {
			self.external_laser.clear_button(ui);
			self.external_laser.external_intensity_entry(ui);
		});
		self.external_laser.get_value()
	}
}

pub fn add_heading(ui: &mut egui::Ui) {
	ui.add_space(PREHEADING_SPACE);
	ui.label(
		egui::RichText::new("Dataset Assistant")
			.size(HEADING_FONT_SIZE)
			.extra_letter_spacing(HEADING_FONT_SPACING),
	);
	ui.separator();
}

pub fn add_subsection<F, R>(
	ui: &mut egui::Ui,
	subtitle: &str,
	add_contents: F,
) -> egui::collapsing_header::CollapsingResponse<R>
where
	F: FnOnce(&mut egui::Ui) -> R,
{
	ui.add_space(5.0);
	let rich_text = egui::RichText::new(subtitle)
		.size(SUBHEADING_FONT_SIZE)
		.extra_letter_spacing(SUBHEADING_FONT_SPACING);
	egui::CollapsingHeader::new(rich_text)
		.default_open(true)
		.icon(|_, _, _| {})
		.show(ui, add_contents)
}

/// First option is whether the menu is open.
/// Second option is whether a button was clicked.
/// Third option is the index.
#[expect(clippy::option_option, reason = "It works okay")]
pub fn camera_selection(
	ui: &mut egui::Ui,
	state: &str,
) -> Option<Option<Option<u32>>> {
	ui.horizontal(|ui| {
		ui.label("Selected ID:");
		ui.label(state);

		ui.menu_button("Select camera", |ui| {
			if ui.button("None").clicked() {
				return Some(None);
			}

			// Cannot be cached because state might change.
			for info in query_for_available_cameras() {
				let Ok(index) = info.index().as_index() else {
					continue;
				};
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

#[expect(clippy::option_option, reason = "It works okay")]
pub fn build_csv_selection(ui: &mut egui::Ui) -> Option<Option<u8>> {
	ui.horizontal(|ui| {
		ui.menu_button("Build CSV", |ui| {
			if ui.button("Standard").clicked() {
				ui.close();
				return Some(1);
			}

			if ui.button("Legacy").clicked() {
				ui.close();
				return Some(0);
			}

			None
		})
		.inner
	})
	.inner
}

pub fn capture_controls(ui: &mut egui::Ui, live_feed: &mut bool) -> bool {
	ui.horizontal(|ui| {
		let btn_result =
			ui.add_enabled(!*live_feed, egui::Button::new("Preview capture"));
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

	let trigger = res.iter().any(eframe::egui::Response::lost_focus);
	let none_empty =
		!camera_position_input.is_empty() && !laser_intensity_input.is_empty();

	(trigger && none_empty).then(|| {
		Subset::new(
			camera_position_input.clone(),
			laser_intensity_input.clone(),
		)
	})
}

/// Draw a section for a single control.
///
/// IDEA: Could use [`egui::DragValue`] or [`egui::Slider`]. Would require a
/// large amount of work to validate data before applying it.
fn draw_for_control(
	ui: &mut egui::Ui,
	manager: &mut ControlManager,
) -> Option<ConValueIntegerType> {
	let default_label = "def:".to_owned();
	let current_label = "cur:".to_owned();
	let valstrs = [
		default_label.clone(),
		manager.control_default().to_string(),
		current_label.clone(),
		manager.control_value().to_string(),
	];
	let minstrs = [
		default_label.clone(),
		manager.control_min().to_string(),
		current_label.clone(),
		manager.min.to_string(),
	];
	let maxstrs = [
		default_label.clone(),
		manager.control_max().to_string(),
		current_label.clone(),
		manager.max.to_string(),
	];
	let stepstrs = [
		default_label.clone(),
		manager.control_step().to_string(),
		current_label.clone(),
		manager.step.to_string(),
	];
	let entries = [
		OnDemandTextEntryBox::new(
			String::from("Value:"),
			valstrs,
			&mut manager.value_input,
		),
		OnDemandTextEntryBox::new(
			String::from("Min:"),
			minstrs,
			&mut manager.min_input,
		),
		OnDemandTextEntryBox::new(
			String::from("Max:"),
			maxstrs,
			&mut manager.max_input,
		),
		OnDemandTextEntryBox::new(
			String::from("Step:"),
			stepstrs,
			&mut manager.step_input,
		),
	];

	ui.checkbox(&mut manager.enable, "Active");

	let grid_label = manager.base.control_id().to_string();
	let res = ui
		.add_enabled_ui(manager.enable, |ui| {
			input_grid_parsed_const(&grid_label, entries, ui)
		})
		.inner;

	if let Some(new_min) = res[1] {
		manager.set_min(new_min);
	}

	if let Some(new_max) = res[2] {
		manager.set_max(new_max);
	}

	if let Some(new_step) = res[3] {
		manager.set_step(new_step);
	}

	match res[0] {
		Some(new_val) => {
			_ = manager.try_set_control_value(new_val);
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
		.iter_mut()
		.map(|control_ref| {
			let name = control_ref.base.control_id().to_string();
			ui.collapsing(name, |ui| draw_for_control(ui, control_ref))
				.body_returned
				.flatten()
		})
		.collect();

	res.iter().any(Option::is_some).then_some(res)
}

pub fn save_button(ui: &mut egui::Ui, enabled: bool) -> bool {
	let save_btn = egui::Button::new("Run save procedure");
	let response = ui.add_enabled(enabled, save_btn);
	response.clicked()
}

pub fn draw_collecting(ui: &mut egui::Ui, progress: &str) {
	ui.label(progress);
}

pub fn draw_save_estimate(
	ui: &mut egui::Ui,
	permutations: u128,
	duration: Duration,
) {
	let seconds = duration.as_secs();
	let hours = seconds.div_euclid(3_600);
	let minutes = (seconds % 3_600).div_euclid(60);
	let seconds = seconds % 60;
	ui.label(format!("Permutations: {permutations}"));
	ui.label(format!(
		"Estimated duration: {hours}h {minutes}m {seconds}s (approx.)"
	));
}

pub fn display_panel<F, R>(
	ui: &mut egui::Ui,
	add_contents: F,
) -> egui::InnerResponse<R>
where
	F: FnOnce(&mut egui::Ui) -> R,
{
	let frame = create_frame(ui);
	egui::CentralPanel::default_margins()
		.frame(frame)
		.show(ui, add_contents)
}

pub fn control_panel_normal<F, R>(
	ui: &mut egui::Ui,
	add_contents: F,
) -> egui::InnerResponse<R>
where
	F: FnOnce(&mut egui::Ui) -> R,
{
	let frame = create_frame(ui);
	egui::CentralPanel::default_margins()
		.frame(frame)
		.show(ui, |ui| control_panel_contents(ui, add_contents).inner)
}

pub fn control_panel_expanded<F, R>(
	ui: &mut egui::Ui,
	add_contents: F,
) -> egui::InnerResponse<R>
where
	F: FnOnce(&mut egui::Ui) -> R,
{
	let frame = create_frame(ui);
	egui::Panel::left("controls")
		.resizable(true)
		.min_size(CONTROLS_MIN_WIDTH)
		.max_size(CONTROLS_MAX_WIDTH)
		.frame(frame)
		.show(ui, |ui| control_panel_contents(ui, add_contents).inner)
}

fn create_frame(ui: &mut egui::Ui) -> egui::Frame {
	let fill = ui.style().visuals.panel_fill;
	let grey = egui::Color32::from_gray(FRAME_STROKE_GREY);
	let stroke = egui::Stroke::new(FRAME_STROKE_WIDTH, grey);
	egui::Frame::new()
		.inner_margin(FRAME_INNER_MARGIN)
		.fill(fill)
		.stroke(stroke)
}

fn control_panel_contents<F, R>(
	ui: &mut egui::Ui,
	add_contents: F,
) -> egui::scroll_area::ScrollAreaOutput<R>
where
	F: FnOnce(&mut egui::Ui) -> R,
{
	ui.style_mut().spacing.scroll = egui::style::ScrollStyle::solid();
	egui::ScrollArea::vertical().show(ui, add_contents)
}

pub fn adjust_viewport_expanded(ui: &mut egui::Ui) {
	// TODO: Clamp to current control width + image width.
	set_viewport_min_max(
		ui,
		[EXPANDED_MIN_WIDTH, EXPANDED_MIN_HEIGHT],
		[EXPANDED_MAX_WIDTH, EXPANDED_MAX_HEIGHT],
	);
	let new_width = ui.viewport_rect().width() + DISPLAY_MIN_WIDTH;
	set_viewport_size(ui, [new_width, ui.viewport_rect().height()]);
}

pub fn adjust_viewport_condensed(ui: &mut egui::Ui) {
	set_viewport_min_max(
		ui,
		[CONTROLS_MIN_WIDTH, DISPLAY_MIN_HEIGHT],
		[CONTROLS_MAX_WIDTH, 0.0],
	);
	let new_width = ui.viewport_rect().width() - DISPLAY_MIN_WIDTH;
	let new_width = new_width.clamp(CONTROLS_MIN_WIDTH, CONTROLS_MAX_WIDTH);
	set_viewport_size(ui, [new_width, ui.viewport_rect().height()]);
}

fn set_viewport_min_max(
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
