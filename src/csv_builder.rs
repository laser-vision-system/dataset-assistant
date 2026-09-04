use std::{error::Error, fmt, path::PathBuf, thread, time::Duration};

pub struct CSVBuilder {
	handle: Option<JoinHandle>,
}

impl CSVBuilder {
	pub fn new(path: PathBuf) -> Self {
		let handle = thread::Builder::new()
			.name("csv_builder".into())
			.spawn(move || run(path))
			.unwrap();
		Self {
			handle: Some(handle),
		}
	}

	pub fn is_done(&mut self) -> Option<CSVBuilderResult> {
		let Some(handle) = self.handle.take() else {
			return None;
		};

		if handle.is_finished() {
			return Some(handle.into());
		} else {
			self.handle.replace(handle);
			None
		}
	}
}

type JoinHandle = thread::JoinHandle<Result<()>>;

#[derive(Debug)]
pub struct CSVBuilderError {
	why: String,
}

impl CSVBuilderError {
	fn new(why: String) -> Self {
		Self { why }
	}
}

impl fmt::Display for CSVBuilderError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "waaaa")
	}
}

impl Error for CSVBuilderError {}

type Result<T> = std::result::Result<T, CSVBuilderError>;

pub struct CSVBuilderResult {
	handle: JoinHandle,
}

impl CSVBuilderResult {
	pub fn join(self) -> Result<()> {
		self.handle
			.join()
			.expect("thread panicked and I don't know how to handle it")
	}
}

impl From<JoinHandle> for CSVBuilderResult {
	fn from(handle: JoinHandle) -> Self {
		Self { handle }
	}
}

fn run(path: PathBuf) -> Result<()> {
	println!("{}", path.display());
	thread::sleep(Duration::from_secs(2));
	Ok(())
}
