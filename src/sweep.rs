use std::{fmt, iter::StepBy, ops::RangeInclusive};

use edge::tcam_interface::{
	CameraDevice,
	controls::{
		traits::{ControlDefault, ControlDescription},
		values::{ConValueInteger, ConValueIntegerType},
	},
};

use crate::camera_controls::ControlManager;

pub type Finished = bool;

#[derive(Debug)]
pub struct ControlSweep {
	control: ControlManager,
	/// The iterator actively being consumed.
	active_iter: StepBy<RangeInclusive<ConValueIntegerType>>,
}

impl ControlSweep {
	pub fn step(&mut self) -> Finished {
		if let Some(val) = self.active_iter.next() {
			self.control
				.set_value(val)
				.expect("control should always be set to a valid value");
			false
		} else {
			self.active_iter = Self::create_range(&self.control);
			self.update_value();
			true
		}
	}

	fn update_value(&mut self) {
		self.control
			.set_value(self.active_iter.next().unwrap().into())
			.expect("control should always be set to a valid value");
	}

	pub fn to_control_type_value(&self) -> ConValueInteger {
		self.control.base.get_value()
	}

	fn create_range(manager: &ControlManager) -> StepBy<RangeInclusive<ConValueIntegerType>> {
		(manager.min..=manager.max).step_by(manager.step as usize)
	}

	pub fn apply_default(&mut self, camera: &mut CameraDevice) {
		self.control.reset_to_default();
		let id = self.control.get_id();
		let value = self.control.base.get_value();
		camera
			.set_control(id, value)
			.expect("control should always be set to a valid value");
	}

	pub fn apply_current(&mut self, camera: &mut CameraDevice) {
		let id = self.control.get_id();
		let value = self.control.base.get_value();
		camera
			.set_control(id, value)
			.expect("control should always be set to a valid value");
	}
}

impl From<ControlManager> for ControlSweep {
	fn from(value: ControlManager) -> Self {
		let active_iter = Self::create_range(&value);
		let mut s = Self {
			control: value,
			active_iter,
		};
		s.update_value();
		s
	}
}

impl fmt::Display for ControlSweep {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}: {}", self.control.get_id(), self.control.get_value())
	}
}
