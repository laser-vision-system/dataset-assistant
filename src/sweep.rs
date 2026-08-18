use std::{fmt, iter::StepBy, ops::RangeInclusive};

use edge::Camera;

use nokhwa::utils::{ControlValueSetter, KnownCameraControl};

pub type Finished = bool;

pub trait ValidControlValue: Copy + fmt::Display {}

impl ValidControlValue for i64 {}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ControlTypeValue<T>
where
	T: ValidControlValue,
{
	control_type: KnownCameraControl,
	current_value: T,
}

impl<T> ControlTypeValue<T>
where
	T: ValidControlValue,
{
	pub fn new(control_type: KnownCameraControl, current_value: T) -> Self {
		Self {
			control_type,
			current_value,
		}
	}

	pub fn current_value(&self) -> T {
		self.current_value
	}
}

impl<T> fmt::Display for ControlTypeValue<T>
where
	T: ValidControlValue,
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}: {}", self.control_type, self.current_value)
	}
}

#[derive(Clone, Debug)]
pub struct ControlSweep<T>
where
	T: ValidControlValue,
{
	control_type: KnownCameraControl,
	/// Range values are inclusive at both ends.
	control_range: StepBy<RangeInclusive<T>>,
	/// The iterator actively being consumed.
	active_iter: StepBy<RangeInclusive<T>>,
	/// Access to the current value without advancing.
	current_value: T,
	default: T,
}

impl<T> ControlSweep<T>
where
	T: ValidControlValue,
	RangeInclusive<T>: Iterator<Item = T>,
{
	pub fn new(
		control_type: KnownCameraControl,
		control_range: StepBy<RangeInclusive<T>>,
		default: T,
	) -> Self {
		let mut active_iter = control_range.clone();
		let current_value = active_iter.next().unwrap();
		Self {
			control_type,
			control_range,
			active_iter,
			current_value,
			default,
		}
	}

	pub fn step(&mut self) -> Finished {
		if let Some(val) = self.active_iter.next() {
			self.current_value = val;
			false
		} else {
			self.active_iter = self.control_range.clone();
			self.current_value = self.active_iter.next().unwrap();
			true
		}
	}

	pub fn to_control_type_value(&self) -> ControlTypeValue<T> {
		ControlTypeValue::new(self.control_type, self.current_value)
	}
}

impl ControlSweep<i64> {
	pub fn set_to_default(&self, camera: &mut Camera) {
		let value = ControlValueSetter::Integer(self.default);
		camera.set_control(self.control_type, value).unwrap();
	}

	pub fn set_to_value(&self, camera: &mut Camera) {
		let setter = ControlValueSetter::Integer(self.current_value);
		camera.set_control(self.control_type, setter).unwrap();
	}
}

impl<T> fmt::Display for ControlSweep<T>
where
	T: ValidControlValue,
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}: {}", self.control_type, self.current_value)
	}
}

#[derive(Clone, Debug)]
pub enum ControlSweepType {
	Brightness(ControlSweep<i64>),
	Contrast(ControlSweep<i64>),
	Saturation(ControlSweep<i64>),
	Hue(ControlSweep<i64>),
	Gamma(ControlSweep<i64>),
	Gain(ControlSweep<i64>),
	//Sharpness(ControlSweep<i64>), // Doesn't seem to do anything.
	ExposureAbsolute(ControlSweep<i64>),
}

impl ControlSweepType {
	pub fn step(&mut self) -> Finished {
		match self {
			Self::Brightness(inner) => inner.step(),
			Self::Contrast(inner) => inner.step(),
			Self::Saturation(inner) => inner.step(),
			Self::Hue(inner) => inner.step(),
			Self::Gamma(inner) => inner.step(),
			Self::Gain(inner) => inner.step(),
			Self::ExposureAbsolute(inner) => inner.step(),
		}
	}

	pub fn apply_current(&self, camera: &mut Camera) {
		match self {
			Self::Brightness(inner) => {
				inner.set_to_value(camera);
			},
			Self::Contrast(inner) => {
				inner.set_to_value(camera);
			},
			Self::Saturation(inner) => {
				inner.set_to_value(camera);
			},
			Self::Hue(inner) => {
				inner.set_to_value(camera);
			},
			Self::Gamma(inner) => {
				inner.set_to_value(camera);
			},
			Self::Gain(inner) => {
				inner.set_to_value(camera);
			},
			Self::ExposureAbsolute(inner) => {
				inner.set_to_value(camera);
			},
		}
	}

	pub fn apply_default(&self, camera: &mut Camera) {
		match self {
			Self::Brightness(inner) => inner.set_to_default(camera),
			Self::Contrast(inner) => inner.set_to_default(camera),
			Self::Saturation(inner) => inner.set_to_default(camera),
			Self::Hue(inner) => inner.set_to_default(camera),
			Self::Gamma(inner) => inner.set_to_default(camera),
			Self::Gain(inner) => inner.set_to_default(camera),
			Self::ExposureAbsolute(inner) => inner.set_to_default(camera),
		}
	}

	pub fn to_control_type_value(&self) -> ControlTypeValue<i64> {
		match self {
			Self::Brightness(inner) => {
				inner.to_control_type_value()
				//let val = inner
				//	.current()
				//	.map_or_else(|| "None".to_string(), |f| f.to_string());
				//(inner.control_type.to_string(), val)
			},
			Self::Contrast(inner) => inner.to_control_type_value(),
			Self::Saturation(inner) => inner.to_control_type_value(),
			Self::Hue(inner) => inner.to_control_type_value(),
			Self::Gamma(inner) => inner.to_control_type_value(),
			Self::Gain(inner) => inner.to_control_type_value(),
			Self::ExposureAbsolute(inner) => inner.to_control_type_value(),
		}
	}
}

impl fmt::Display for ControlSweepType {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let display_string = match self {
			Self::Brightness(inner) => inner.to_string(),
			Self::Contrast(inner) => inner.to_string(),
			Self::Saturation(inner) => inner.to_string(),
			Self::Hue(inner) => inner.to_string(),
			Self::Gamma(inner) => inner.to_string(),
			Self::Gain(inner) => inner.to_string(),
			Self::ExposureAbsolute(inner) => inner.to_string(),
		};
		write!(f, "{}", display_string)
	}
}
