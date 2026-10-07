//! Execute an operation on a separate thread.

use std::{error::Error, fmt};

pub type RunnerJoinHandle = std::thread::JoinHandle<Result<()>>;
pub type Result<T> = std::result::Result<T, RunnerError>;

/// Types that implement this trait can execute on a separate thread.
pub trait Runner {
	fn handle(&mut self) -> &mut Option<RunnerJoinHandle>;

	fn is_done(&mut self) -> Option<RunnerResult> {
		let handle = self.handle().take()?;

		if handle.is_finished() {
			return Some(handle.into());
		}

		self.handle().replace(handle);
		None
	}
}

#[derive(Debug)]
pub struct RunnerResult {
	handle: RunnerJoinHandle,
}

impl RunnerResult {
	#[expect(
		clippy::unwrap_in_result,
		reason = "This outcome is bad so it should panic"
	)]
	pub fn join(self) -> Result<()> {
		self.handle
			.join()
			.expect("thread panicked and I don't know how to handle it")
	}
}

impl From<RunnerJoinHandle> for RunnerResult {
	fn from(handle: RunnerJoinHandle) -> Self {
		Self { handle }
	}
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct RunnerError {
	why: String,
}

impl RunnerError {
	pub fn new<W>(why: W) -> Self
	where
		W: Into<String>,
	{
		Self { why: why.into() }
	}
}

impl fmt::Display for RunnerError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "CSVBuilderError: {}", self.why)
	}
}

impl Error for RunnerError {}
