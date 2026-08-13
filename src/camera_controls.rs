use edge::Camera;

use nokhwa::utils::{ControlValueDescription, KnownCameraControl};

use crate::sweep::{ControlSweep, ControlSweepType};

pub fn build_control_sweeps(camera: &mut Camera) -> Vec<ControlSweepType> {
	const CONTROLS_OF_INTEREST: [KnownCameraControl; 6] = [
		KnownCameraControl::Brightness,
		KnownCameraControl::Contrast,
		KnownCameraControl::Saturation,
		KnownCameraControl::Hue,
		KnownCameraControl::Gamma,
		KnownCameraControl::Gain,
		//KnownCameraControl::Sharpness, // Doesn't seem to do anything.
	];

	let available_controls = camera.get_all_available_controls();
	let mut variants = Vec::with_capacity(available_controls.len());

	for control in available_controls {
		let control_type = control.control();
		if !CONTROLS_OF_INTEREST.contains(&control_type) {
			continue;
		}
		match *control.description() {
			ControlValueDescription::IntegerRange {
				min,
				max,
				step,
				default,
				..
			} => {
				let control_sweep = ControlSweep::new(
					control_type,
					(min..=max).step_by(step as usize * 5),
					default,
				);
				let control_variant = match control_type {
					KnownCameraControl::Brightness => ControlSweepType::Brightness(control_sweep),
					KnownCameraControl::Contrast => ControlSweepType::Contrast(control_sweep),
					KnownCameraControl::Saturation => ControlSweepType::Saturation(control_sweep),
					KnownCameraControl::Hue => ControlSweepType::Hue(control_sweep),
					KnownCameraControl::Gamma => ControlSweepType::Gamma(control_sweep),
					KnownCameraControl::Gain => ControlSweepType::Gain(control_sweep),
					_ => unimplemented!(),
				};
				variants.push(control_variant);
			},
			_ => panic!("camera control has an unsupported value description"),
		};
	}

	variants
}
