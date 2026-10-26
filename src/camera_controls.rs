use std::time::Duration;

use edge_cameras::{
	camera_device::{
		CameraDevice,
		CameraLike,
	},
	controls::{
		descriptions::{
			CamControl,
			IntegerRangeControl,
		},
		traits::{
			ControlDefault,
			ControlDescription,
			ControlItems as _,
			ControlMax,
			ControlMin,
			ControlStep,
			ControlValue as _,
		},
		values::ConValueIntegerType,
	},
	error::{
		EdgeCamerasError,
		EdgeCamerasResult,
	},
};
use nokhwa::utils::{
	KnownCameraControl,
	MenuItem,
};

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
}

impl ControlManager {
	/// Set the minimum value that the sweep will start at.
	pub fn set_min(&mut self, min: ConValueIntegerType) {
		if min <= self.max
			&& self.control_min() <= min
			&& (self.control_default().saturating_sub(min))
				.checked_rem_euclid(self.control_step())
				.is_some_and(|x| x == 0)
		{
			self.min = min;
		}
	}

	/// Set the maximum value that the sweep is allowed to reach.
	pub fn set_max(&mut self, max: ConValueIntegerType) {
		if self.min <= max
			&& max <= self.control_max()
			&& (self.control_default().saturating_sub(max))
				.checked_rem_euclid(self.control_step())
				.is_some_and(|x| x == 0)
		{
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
		})
	}
}

pub fn try_disable_auto_exposure(
	camera: &mut CameraDevice,
) -> EdgeCamerasResult<()> {
	let id = KnownCameraControl::AutoExposure;
	let CamControl::Menu(control) = camera.get_control(id)? else {
		return Err(EdgeCamerasError::GetControlType(id));
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
		if !matches!(&cam_control, CamControl::IntegerRange(_)) {
			continue;
		}
		let control = ControlManager::try_from(cam_control)
			.expect("camera control has an unsupported value description");
		variants.push(control);
	}

	variants
}

/// Extract current values for camera controls not represented by integer
/// range sweeps.
pub fn extract_fixed_control_values(
	camera: &mut CameraDevice,
	swept_controls: &[KnownCameraControl],
) -> Vec<(String, String)> {
	let Ok(controls) = camera.get_controls() else {
		return Vec::new();
	};
	controls
		.into_inner()
		.into_iter()
		.filter(|(id, control)| {
			!(matches!(control, CamControl::None)
				|| swept_controls.contains(id))
		})
		.flat_map(|(id, control)| {
			[
				control
					.try_display_control_value()
					.map(|label| (id.to_string(), label)),
				control
					.try_display_control_default()
					.map(|label| (format!("{id}-default"), label)),
				control
					.try_display_control_min()
					.map(|label| (format!("{id}-min"), label)),
				control
					.try_display_control_max()
					.map(|label| (format!("{id}-max"), label)),
				control
					.try_display_control_step()
					.map(|label| (format!("{id}-step"), label)),
				control
					.try_display_control_possible()
					.map(|label| (format!("{id}-possible"), label)),
				control
					.try_display_control_items()
					.map(|label| (format!("{id}-items"), label)),
			]
			.into_iter()
			.flatten()
			.collect::<Vec<_>>()
		})
		.collect()
}

pub fn build_control_sweeps(controls: &[ControlManager]) -> Vec<ControlSweep> {
	controls
		.iter()
		.map(|control| control.clone().into())
		.collect()
}

/// Return the number of capture permutations represented by the controls.
pub fn total_permutations(controls: &[ControlManager]) -> u128 {
	controls
		.iter()
		.filter(|control| control.enable)
		.map(|control| {
			let distance = control.max - control.min;
			let count = distance.div_euclid(control.step) + 1;
			u128::try_from(count).unwrap_or(u128::MAX)
		})
		.fold(1, u128::saturating_mul)
}

/// Estimate the save duration from sweep count and exposure value.
///
/// The estimate treats exposure values as milliseconds and includes the
/// fixed camera settling delay. Backends may use different exposure units.
pub fn estimate_save_duration(controls: &[ControlManager]) -> Duration {
	let exposure_ms = controls
		.iter()
		.find(|control| {
			control.control_id() == KnownCameraControl::ExposureAbsolute
		})
		.map_or(0, |control| {
			let value = if control.enable {
				control.max
			} else {
				control.control_default()
			};
			u64::try_from(value.max(0)).unwrap_or(u64::MAX)
		});
	let per_capture_ms = exposure_ms.saturating_add(100);
	let permutation_count =
		u64::try_from(total_permutations(controls)).unwrap_or(u64::MAX);
	Duration::from_millis(per_capture_ms.saturating_mul(permutation_count))
}

#[cfg(test)]
mod tests {
	use edge_cameras::controls::{
		descriptions::IntegerRangeControl,
		values::ConValueInteger,
	};
	use nokhwa::utils::KnownCameraControl;

	use super::*;

	fn control(min: i64, max: i64, step: i64, enable: bool) -> ControlManager {
		let base = IntegerRangeControl::new(
			KnownCameraControl::Brightness,
			ConValueInteger::from(min),
			min,
			step,
			min,
			max,
		);
		ControlManager {
			base,
			value: min,
			step,
			min,
			max,
			enable,
		}
	}

	#[test]
	fn permutation_count_includes_pinned_and_ignores_disabled_controls() {
		let controls = [
			control(4, 4, 1, true),
			control(0, 8, 2, true),
			control(0, 10, 1, false),
		];
		assert_eq!(total_permutations(&controls), 5);
	}
}
