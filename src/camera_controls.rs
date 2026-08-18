use edge::{
	Camera,
	tcam_interface::controls::{
		descriptions::CamControl,
		traits::{ControlDefault, ControlMax, ControlMin, ControlStep},
	},
};

use nokhwa::utils::KnownCameraControl;

use crate::sweep::{ControlSweep, ControlSweepType};

pub fn build_control_sweeps(camera: &mut Camera) -> Vec<ControlSweepType> {
	const CONTROLS_OF_INTEREST: [KnownCameraControl; 7] = [
		KnownCameraControl::Brightness,
		KnownCameraControl::Contrast,
		KnownCameraControl::Saturation,
		KnownCameraControl::Hue,
		KnownCameraControl::Gamma,
		KnownCameraControl::Gain,
		//KnownCameraControl::Sharpness, // Doesn't seem to do anything.
		KnownCameraControl::ExposureAbsolute,
		//KnownCameraControl::AutoExposure, // Menu type not supported.
	];

	let available_controls = camera.get_controls();
	let mut variants = Vec::with_capacity(available_controls.len());

	for (id, control) in available_controls {
		if !CONTROLS_OF_INTEREST.contains(&id) {
			continue;
		}
		match control {
			CamControl::IntegerRange(control) => {
				let control_sweep = ControlSweep::new(
					id,
					(control.get_min()..=control.get_max())
						.step_by(control.get_step() as usize * 5),
					control.get_default(),
				);
				let control_variant = match id {
					KnownCameraControl::Brightness => ControlSweepType::Brightness(control_sweep),
					KnownCameraControl::Contrast => ControlSweepType::Contrast(control_sweep),
					KnownCameraControl::Saturation => ControlSweepType::Saturation(control_sweep),
					KnownCameraControl::Hue => ControlSweepType::Hue(control_sweep),
					KnownCameraControl::Gamma => ControlSweepType::Gamma(control_sweep),
					KnownCameraControl::Gain => ControlSweepType::Gain(control_sweep),
					KnownCameraControl::ExposureAbsolute => {
						ControlSweepType::ExposureAbsolute(control_sweep)
					},
					_ => unimplemented!(),
				};
				variants.push(control_variant);
			},
			_ => panic!("camera control has an unsupported value description"),
		};
	}

	variants
}
