use std::{
	fmt,
	iter::StepBy,
	ops::{ControlFlow, RangeInclusive},
};

use edge_cameras::{
	camera_device::{CameraDevice, CameraLike},
	controls::{
		traits::{ControlDefault as _, ControlDescription as _},
		values::{ConValueInteger, ConValueIntegerType},
	},
};

use crate::camera_controls::ControlManager;

#[derive(Debug)]
pub struct ControlSweep {
	control: ControlManager,
	/// The iterator actively being consumed.
	active_iter: StepBy<RangeInclusive<ConValueIntegerType>>,
}

impl ControlSweep {
	/// Step once through the control sweep.
	///
	/// This returns [`ControlFlow::Continue`] until the iterator is exhausted,
	/// where it returns [`ControlFlow::Break`].
	pub fn step(&mut self) -> ControlFlow<()> {
		if let Some(val) = self.active_iter.next() {
			self.control
				.set_value(val)
				.expect("control should always be set to a valid value");
			ControlFlow::Continue(())
		} else {
			self.active_iter = Self::create_range(&self.control);
			self.update_value();
			ControlFlow::Break(())
		}
	}

	#[expect(
		clippy::unwrap_used,
		reason = "This will only be called when a new active iter is made"
	)]
	fn update_value(&mut self) {
		self.control
			.set_value(self.active_iter.next().unwrap())
			.expect("control should always be set to a valid value");
	}

	pub fn to_control_type_value(&self) -> ConValueInteger {
		self.control.base.get_value()
	}

	#[expect(
		clippy::cast_sign_loss,
		reason = "Step will never be a negative number"
	)]
	#[expect(
		clippy::cast_possible_truncation,
		reason = "Step size will never be big enough"
	)]
	fn create_range(
		manager: &ControlManager,
	) -> StepBy<RangeInclusive<ConValueIntegerType>> {
		(manager.min..=manager.max).step_by(manager.step as usize)
	}

	pub fn apply_default(&mut self, camera: &mut CameraDevice) {
		self.control.reset_to_default();
		let id = self.control.get_id();
		let value = self.control.base.get_value();
		camera
			.set_control(id, value.into())
			.expect("control should always be set to a valid value");
	}

	pub fn apply_current(&mut self, camera: &mut CameraDevice) {
		let id = self.control.get_id();
		let value = self.control.base.get_value();
		camera
			.set_control(id, value.into())
			.expect("control should always be set to a valid value");
	}
}

impl From<ControlManager> for ControlSweep {
	fn from(value: ControlManager) -> Self {
		let active_iter = Self::create_range(&value);
		let mut new = Self {
			control: value,
			active_iter,
		};
		new.update_value();
		new
	}
}

impl fmt::Display for ControlSweep {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}: {}", self.control.get_id(), self.control.get_value())
	}
}
