use edge_cameras::{
	camera_device::{CameraDevice, CameraLike},
	controls::{
		descriptions::{CamControl, IntegerRangeControl},
		traits::{
			ControlDefault, ControlDescription, ControlItems as _, ControlMax,
			ControlMin, ControlStep, ControlValue as _,
		},
		values::ConValueIntegerType,
	},
	error::{EdgeCamerasError, EdgeCamerasResult},
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

const PATTERNS: [&str; 2] = ["manual", "Manual"];

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
		if self.control_min() <= min && min < self.max {
			self.min = min;
		}
	}

	pub fn set_max(&mut self, max: ConValueIntegerType) {
		if self.min < max && max <= self.control_max() {
			self.max = max;
		}
	}

	fn abs_deviation_from_default(
		&self,
		x: ConValueIntegerType,
	) -> ConValueIntegerType {
		(x - self.control_default()).abs()
	}

	pub fn set_step(&mut self, step: ConValueIntegerType) {
		if step <= 0 {
			return;
		}

		let min_deviation = self.abs_deviation_from_default(self.control_min());
		let max_deviation = self.abs_deviation_from_default(self.control_max());
		let total_range = min_deviation + max_deviation;
		if step > total_range {
			return;
		}

		let multiple_of_base_step = step % self.control_step() == 0;
		if multiple_of_base_step {
			self.step = step;
		}
	}
}

impl ControlDescription for ControlManager {
	type ConType = ConValueIntegerType;
	type ValueType = ConValueIntegerType;

	fn control_id(&self) -> KnownCameraControl {
		self.base.control_id()
	}

	fn try_set_control_value(
		&mut self,
		value: Self::ValueType,
	) -> EdgeCamerasResult<()> {
		self.base.try_set_control_value(value.into())?;
		self.value = value;
		Ok(())
	}

	fn control_value(&self) -> Self::ValueType {
		self.value
	}
}

impl ControlDefault for ControlManager {
	type DefaultType = Self::ConType;

	fn control_default(&self) -> Self::DefaultType {
		self.base.control_default()
	}
}

impl ControlStep for ControlManager {
	type StepType = Self::ConType;

	fn control_step(&self) -> Self::StepType {
		self.base.control_step()
	}

	fn validate(&self, value: Self::StepType) -> bool {
		(value - self.control_default()).abs() % self.control_step() == 0
			|| (value - self.control_value()).abs() % self.control_step() == 0
	}
}

impl ControlMin for ControlManager {
	type MinType = Self::ConType;

	fn control_min(&self) -> Self::MinType {
		self.base.control_min()
	}
}

impl ControlMax for ControlManager {
	type MaxType = Self::ConType;

	fn control_max(&self) -> Self::MaxType {
		self.base.control_max()
	}
}

impl TryFrom<CamControl> for ControlManager {
	type Error = EdgeCamerasError;

	fn try_from(value: CamControl) -> EdgeCamerasResult<Self> {
		let CamControl::IntegerRange(control) = value else {
			return Err(EdgeCamerasError::InvalidControlValue);
		};
		Ok(Self {
			base: control,
			value: control.control_value().get(),
			step: control.control_step(),
			min: control.control_min(),
			max: control.control_max(),
			enable: true,
			value_input: String::default(),
			step_input: String::default(),
			min_input: String::default(),
			max_input: String::default(),
		})
	}
}

pub fn try_disable_auto_exposure(
	camera: &mut CameraDevice,
) -> EdgeCamerasResult<()> {
	let id = KnownCameraControl::AutoExposure;
	let CamControl::Menu(control) = camera.get_control(id)? else {
		return Err(EdgeCamerasError::from(EdgeCamerasError::GetControlType(
			id,
		)));
	};
	let items = control.control_items();
	let valid_entires: Vec<_> = items
		.iter()
		.filter(|&entry| {
			let MenuItem::Name(name) = entry.get_item() else {
				return false;
			};
			PATTERNS.into_iter().any(|pattern| name.contains(pattern))
		})
		.collect();
	let &first = valid_entires
		.first()
		.ok_or(EdgeCamerasError::InvalidControlValue)?;
	let mut value = control.control_value();
	*value.get_mut() = first.get_id();
	#[expect(
		clippy::unwrap_in_result,
		reason = "value should be validated by control description"
	)]
	camera
		.set_control(id, value.into())
		.expect("value should be validated by control description");
	Ok(())
}

pub fn extract_controls(
	camera: &mut CameraDevice,
	skip_exposure: bool,
) -> Vec<ControlManager> {
	let available_controls =
		camera.get_controls().expect("always have controls");
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

pub fn build_control_sweeps(controls: &[ControlManager]) -> Vec<ControlSweep> {
	controls
		.iter()
		.filter(|&control| control.enable)
		.map(|control| control.clone().into())
		.collect()
}
