use std::{
	path::PathBuf,
	thread::{self, JoinHandle, sleep},
	time::Duration,
};

pub struct CSVBuilder {
	handle: Option<JoinHandle<()>>,
}

impl CSVBuilder {
	pub fn new(path: PathBuf) -> Self {
		let handle = thread::Builder::new()
			.name("csv_builder".into())
			.spawn(move || {
				println!("{}", path.display());
				sleep(Duration::from_secs(2));
				()
			})
			.unwrap();
		Self {
			handle: Some(handle),
		}
	}

	pub fn is_done(&mut self) -> Option<JoinHandle<()>> {
		let Some(handle) = self.handle.take() else {
			return None;
		};

		if handle.is_finished() {
			return Some(handle);
		} else {
			self.handle.replace(handle);
			None
		}
	}
}
