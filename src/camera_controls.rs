use edge::tcam_interface::{
	CameraDevice,
	controls::{
		descriptions::{CamControl, IntegerRangeControl},
		traits::{
			ControlDefault, ControlDescription, ControlItems, ControlMax, ControlMin, ControlStep,
			ControlValue,
		},
		values::ConValueIntegerType,
	},
};

use nokhwa::utils::{KnownCameraControl, MenuItem};

use crate::sweep::ControlSweep;

const CONTROLS_OF_INTEREST: [KnownCameraControl; 8] = [
	KnownCameraControl::Brightness,
	KnownCameraControl::Contrast,
	KnownCameraControl::Saturation,
	KnownCameraControl::Hue,
	KnownCameraControl::Gamma,
	KnownCameraControl::Gain,
	KnownCameraControl::Sharpness, // Doesn't seem to do anything.
	KnownCameraControl::ExposureAbsolute,
	// TODO: White balance.
];

#[derive(Clone, Debug)]
pub struct ControlManager {
	pub base: IntegerRangeControl,
	pub value: ConValueIntegerType,
	pub step: ConValueIntegerType,
	pub min: ConValueIntegerType,
	pub max: ConValueIntegerType,
	pub enable: bool,
	// These have to remain public because of mutable borrowing rules.
	pub value_input: String,
	pub step_input: String,
	pub min_input: String,
	pub max_input: String,
}

impl ControlManager {
	pub fn set_min(&mut self, min: ConValueIntegerType) {
		if self.get_min() <= min && min < self.max {
			self.min = min;
		}
	}

	pub fn set_max(&mut self, max: ConValueIntegerType) {
		if self.min < max && max <= self.get_max() {
			self.max = max;
		}
	}

	fn abs_deviation_from_default(&self, x: ConValueIntegerType) -> ConValueIntegerType {
		(x - self.get_default()).abs()
	}

	pub fn set_step(&mut self, step: ConValueIntegerType) {
		if step <= 0 {
			return;
		}

		let min_deviation = self.abs_deviation_from_default(self.get_min());
		let max_deviation = self.abs_deviation_from_default(self.get_max());
		let total_range = min_deviation + max_deviation;
		if step > total_range {
			return;
		}

		let multiple_of_base_step = step % self.get_step() == 0;
		if multiple_of_base_step {
			self.step = step;
		}
	}
}

impl ControlDescription for ControlManager {
	type ValueType = ConValueIntegerType;
	type ConType = ConValueIntegerType;

	fn get_id(&self) -> KnownCameraControl {
		self.base.get_id()
	}

	fn set_value(&mut self, value: Self::ValueType) -> Result<(), ()> {
		self.base.set_value(value.into())?;
		self.value = value;
		Ok(())
	}

	fn get_value(&self) -> Self::ValueType {
		self.value
	}
}

impl ControlDefault for ControlManager {
	type DefaultType = Self::ConType;

	fn get_default(&self) -> Self::DefaultType {
		self.base.get_default()
	}

	fn reset_to_default(&mut self) {
		self.set_value(self.get_default().into())
			.expect("default value should never be invalid")
	}
}

impl ControlStep for ControlManager {
	type StepType = Self::ConType;

	fn get_step(&self) -> Self::StepType {
		self.base.get_step()
	}

	fn valid(&self, value: Self::StepType) -> bool {
		(value - self.get_default()).abs() % self.get_step() == 0
			|| (value - self.get_value()).abs() % self.get_step() == 0
	}
}

impl ControlMin for ControlManager {
	type MinType = Self::ConType;

	fn get_min(&self) -> Self::MinType {
		self.base.get_min()
	}
}

impl ControlMax for ControlManager {
	type MaxType = Self::ConType;

	fn get_max(&self) -> Self::MaxType {
		self.base.get_max()
	}
}

impl TryFrom<CamControl> for ControlManager {
	type Error = ();

	fn try_from(value: CamControl) -> Result<Self, Self::Error> {
		let CamControl::IntegerRange(control) = value else {
			return Err(());
		};
		Ok(Self {
			base: control,
			value: control.get_value().get_value(),
			step: control.get_step(),
			min: control.get_min(),
			max: control.get_max(),
			enable: true,
			value_input: Default::default(),
			step_input: Default::default(),
			min_input: Default::default(),
			max_input: Default::default(),
		})
	}
}

pub fn try_disable_auto_exposure(camera: &mut CameraDevice) -> Result<(), ()> {
	let id = KnownCameraControl::AutoExposure;
	let CamControl::Menu(control) = camera.get_control(id) else {
		return Err(());
	};
	let items = control.get_items();
	let valid_entires: Vec<_> = items
		.iter()
		.filter(|&entry| {
			let MenuItem::Name(name) = entry.get_item() else {
				return false;
			};
			const PATTERNS: [&str; 2] = ["manual", "Manual"];
			PATTERNS.into_iter().any(|pattern| name.contains(pattern))
		})
		.collect();
	let &first = valid_entires.first().ok_or(())?;
	let mut value = control.get_value();
	value.set_value(first.get_id());
	camera
		.set_control(id, value)
		.expect("value should be validated by control description");
	Ok(())
}

pub fn extract_controls(camera: &mut CameraDevice, skip_exposure: bool) -> Vec<ControlManager> {
	let available_controls = camera.get_controls();
	let mut variants = Vec::with_capacity(available_controls.len());

	for (id, cam_control) in available_controls.into_inner() {
		if !CONTROLS_OF_INTEREST.contains(&id)
			|| (id == KnownCameraControl::ExposureAbsolute && skip_exposure)
		{
			continue;
		}
		let control = ControlManager::try_from(cam_control)
			.expect("camera control has an unsupported value description");
		variants.push(control);
	}

	variants
}

pub fn build_control_sweeps(controls: &Vec<ControlManager>) -> Vec<ControlSweep> {
	controls
		.iter()
		.filter_map(|control| {
			if control.enable {
				Some(control.clone().into())
			} else {
				None
			}
		})
		.collect()
}
