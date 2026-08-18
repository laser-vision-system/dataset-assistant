use edge::{
	Camera,
	tcam_interface::controls::{
		descriptions::{CamControl, IntegerRangeControl},
		traits::{
			ControlDefault, ControlDescription, ControlItems, ControlMax, ControlMin, ControlStep,
			ControlValue,
		},
		values::ConValueIntegerType,
	},
};

use nokhwa::utils::{KnownCameraControl, MenuItem};

use crate::sweep::{ControlSweep, ControlSweepType};

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

#[derive(Debug)]
pub struct ControlManager {
	pub base: IntegerRangeControl,
	pub value: ConValueIntegerType,
	pub step: ConValueIntegerType,
	pub min: ConValueIntegerType,
	pub max: ConValueIntegerType,
	pub input: String,
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
			input: Default::default(),
		})
	}
}

pub fn try_disable_auto_exposure(camera: &mut Camera) -> Result<(), ()> {
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
		.set_control(id, value.into())
		.expect("value should be validated by control description");
	Ok(())
}

pub fn extract_controls(camera: &mut Camera, skip_exposure: bool) -> Vec<ControlManager> {
	let available_controls = camera.get_controls();
	let mut variants = Vec::with_capacity(available_controls.len());

	for (id, cam_control) in available_controls {
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

pub fn build_control_sweeps(camera: &mut Camera) -> Vec<ControlSweepType> {
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
