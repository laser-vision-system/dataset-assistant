//! A CSV builder of the legacy format.

use std::{
	path::{Path, PathBuf},
	thread,
};

use walkdir::{DirEntry, WalkDir};

use super::{
	FileWriter, get_file_name, get_relative_path, is_image_file, is_not_git_dir,
};
use crate::runner::{Result, Runner, RunnerError, RunnerJoinHandle};

const OUTPUT_NAME: &str = "index.csv";

#[derive(Debug)]
pub struct Builder {
	handle: Option<RunnerJoinHandle>,
}

impl Builder {
	#[expect(
		clippy::unwrap_used,
		reason = "Unlikely to happen, and panic is desired"
	)]
	pub fn new(path: PathBuf) -> Self {
		let handle = thread::Builder::new()
			.name("legacy_csv_builder".into())
			.spawn(move || run(path))
			.unwrap();
		Self {
			handle: Some(handle),
		}
	}
}

impl Runner for Builder {
	fn handle(&mut self) -> &mut Option<RunnerJoinHandle> {
		&mut self.handle
	}
}

/// Defines the identified structure of the legacy dataset.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
enum LegacyFormat {
	Single,
	Double,
}

/// Contains extracted information for a single legacy image dataset entry.
#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct LegacyEntry {
	relative_path: String,
	external_intensity: Option<String>,
	intensity: String,
}

/// Main execution pipeline for the legacy CSV builder thread.
#[expect(clippy::needless_pass_by_value, reason = "This should be consuming")]
fn run(root_path: PathBuf) -> Result<()> {
	let mut entry_stream = WalkDir::new(&root_path)
		.min_depth(1)
		.into_iter()
		.filter_entry(is_not_git_dir)
		.filter_map(std::result::Result::ok)
		.filter(is_image_file);

	let csv_path = root_path.join(OUTPUT_NAME);
	let mut writer = csv::Writer::from_path(csv_path).map_err(|why| {
		RunnerError::new(format!("failed to create csv writer: {why}"))
	})?;

	write_stream(&root_path, &mut writer, &mut entry_stream)?;

	writer
		.flush()
		.map_err(|why| RunnerError::new(format!("failed to flush csv: {why}")))
}

/// Processes the iterator stream, detects the format, then writes headers and
/// all subsequent records.
fn write_stream(
	root: &Path,
	writer: &mut FileWriter,
	stream: &mut impl Iterator<Item = DirEntry>,
) -> Result<()> {
	let Some(first_entry) = stream.next() else {
		return Ok(());
	};

	let format = detect_format(&first_entry)?;
	write_headers(writer, format)?;

	let first_record = create_legacy_entry(root, &first_entry, format)?;
	write_record(writer, &first_record)?;

	for entry in stream {
		let record = create_legacy_entry(root, &entry, format)?;
		write_record(writer, &record)?;
	}

	Ok(())
}

/// Derives the dataset format from the first identified image file.
fn detect_format(entry: &DirEntry) -> Result<LegacyFormat> {
	match get_segments(entry)?.len() {
		2 => Ok(LegacyFormat::Single),
		3 => Ok(LegacyFormat::Double),
		_ => Err(RunnerError::new("unsupported legacy filename format")),
	}
}

/// Split an entry name into segments by underscores.
fn get_segments(entry: &DirEntry) -> Result<Vec<String>> {
	let name = get_file_name(entry.path())?;
	let name_without_ext = name.strip_suffix(".png").unwrap_or(&name);
	let segments: Vec<String> =
		name_without_ext.split('_').map(str::to_owned).collect();
	Ok(segments)
}

/// Writes the header row based on the detected format.
fn write_headers(writer: &mut FileWriter, format: LegacyFormat) -> Result<()> {
	let headers = match format {
		LegacyFormat::Single => vec!["relative_path", "intensity"],
		LegacyFormat::Double => {
			vec!["relative_path", "intensity", "external_intensity"]
		},
	};

	writer.write_record(&headers).map_err(|why| {
		RunnerError::new(format!("failed to write csv headers: {why}"))
	})
}

/// Writes a single dataset entry as a row in the CSV file.
fn write_record(writer: &mut FileWriter, entry: &LegacyEntry) -> Result<()> {
	let mut record = vec![entry.relative_path.clone()];

	record.push(entry.intensity.clone());

	if let Some(ext_intensity) = &entry.external_intensity {
		record.push(ext_intensity.clone());
	}

	writer.write_record(&record).map_err(|why| {
		RunnerError::new(format!("failed to write csv record: {why}"))
	})
}

/// Constructs a legacy entry by parsing the filename according to the detected
/// format.
#[expect(clippy::indexing_slicing, reason = "Length validated prior")]
fn create_legacy_entry(
	root: &Path,
	entry: &DirEntry,
	format: LegacyFormat,
) -> Result<LegacyEntry> {
	let relative_path = get_relative_path(root, entry.path())?;
	let segments = get_segments(entry)?;
	if segments.len() <= 1 {
		return Err(RunnerError::new("not enough segments in filename"));
	}

	match format {
		LegacyFormat::Single => {
			if segments.len() != 2 {
				return Err(RunnerError::new(
					"inconsistent dataset formatting detected",
				));
			}
			Ok(LegacyEntry {
				relative_path,
				external_intensity: None,
				intensity: segments[1].clone(),
			})
		},
		LegacyFormat::Double => {
			if segments.len() != 3 {
				return Err(RunnerError::new(
					"inconsistent dataset formatting detected",
				));
			}
			Ok(LegacyEntry {
				relative_path,
				external_intensity: Some(segments[1].clone()),
				intensity: segments[2].clone(),
			})
		},
	}
}
