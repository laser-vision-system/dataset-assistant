use std::time::Duration;

use rodio::{MixerDeviceSink, Source};

#[derive(Debug)]
pub struct Audio {
	sink: MixerDeviceSink,
}

impl Audio {
	pub fn play_tones(&self, tones: &[Tone]) {
		let player = rodio::Player::connect_new(self.sink.mixer());
		for tone in tones {
			let tone = rodio::source::SineWave::new(tone.frequency)
				.take_duration(tone.duration);
			player.append(tone);
		}
		player.detach();
	}

	pub fn play_save_tone(&self) {
		let tones = [
			Tone::beep(),
			Tone::silence(Duration::from_millis(30)),
			Tone::beep(),
		];
		self.play_tones(&tones);
	}
}

impl Default for Audio {
	fn default() -> Self {
		let mut sink = rodio::DeviceSinkBuilder::open_default_sink()
			.expect("This device should have an audio sink");
		sink.log_on_drop(false);
		Self { sink }
	}
}

#[derive(Debug)]
pub struct Tone {
	frequency: f32,
	duration: Duration,
}

impl Tone {
	pub fn new(frequency: f32, duration: Duration) -> Self {
		Self {
			frequency,
			duration,
		}
	}

	pub fn silence(duration: Duration) -> Self {
		Self {
			frequency: f32::EPSILON,
			duration,
		}
	}

	fn beep() -> Self {
		Self::new(1000.0, Duration::from_millis(60))
	}
}

impl From<(f32, Duration)> for Tone {
	fn from(value: (f32, Duration)) -> Self {
		Self::new(value.0, value.1)
	}
}
