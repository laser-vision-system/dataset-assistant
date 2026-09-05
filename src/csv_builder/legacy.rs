//! A CSV builder of the legacy format.

use std::{
	path::{Path, PathBuf},
	thread,
};
use walkdir::{DirEntry, WalkDir};

use super::{_FileWriter, _JoinHandle, CSVBuilderError, CSVBuilderHandle, Result, common};

const OUTPUT_NAME: &str = "index.csv";

#[derive(Debug)]
pub struct Builder {
	handle: Option<_JoinHandle>,
}

impl Builder {
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

impl CSVBuilderHandle for Builder {
	fn handle(&mut self) -> &mut Option<_JoinHandle> {
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
fn run(root_path: PathBuf) -> Result<()> {
	let mut entry_stream = WalkDir::new(&root_path)
		.min_depth(1)
		.into_iter()
		.filter_entry(common::is_not_git_dir)
		.filter_map(|e| e.ok())
		.filter(common::is_image_file);

	let csv_path = root_path.join(OUTPUT_NAME);
	let mut writer = csv::Writer::from_path(csv_path)
		.map_err(|why| CSVBuilderError::new(format!("failed to create csv writer: {why}")))?;

	write_stream(&root_path, &mut writer, &mut entry_stream)?;

	writer
		.flush()
		.map_err(|why| CSVBuilderError::new(format!("failed to flush csv: {why}")))
}

/// Processes the iterator stream, detects the format, then writes headers and all subsequent records.
fn write_stream(
	root: &Path,
	writer: &mut _FileWriter,
	stream: &mut impl Iterator<Item = DirEntry>,
) -> Result<()> {
	let Some(first_entry) = stream.next() else {
		return Ok(());
	};

	let format = detect_format(&first_entry)?;
	write_headers(writer, &format)?;

	let first_record = create_legacy_entry(root, &first_entry, &format)?;
	write_record(writer, &first_record)?;

	for entry in stream {
		let record = create_legacy_entry(root, &entry, &format)?;
		write_record(writer, &record)?;
	}

	Ok(())
}

/// Derives the dataset format from the first identified image file.
fn detect_format(entry: &DirEntry) -> Result<LegacyFormat> {
	match get_segments(entry)?.len() {
		2 => Ok(LegacyFormat::Single),
		3 => Ok(LegacyFormat::Double),
		_ => Err(CSVBuilderError::new("unsupported legacy filename format")),
	}
}

/// Split an entry name into segments by underscores.
fn get_segments(entry: &DirEntry) -> Result<Vec<String>> {
	let name = common::get_file_name(entry.path())?;
	let name_without_ext = name.strip_suffix(".png").unwrap_or(&name);
	let segments: Vec<String> = name_without_ext
		.split('_')
		.map(ToString::to_string)
		.collect();
	Ok(segments)
}

/// Writes the header row based on the detected format.
fn write_headers(writer: &mut _FileWriter, format: &LegacyFormat) -> Result<()> {
	let headers = match format {
		LegacyFormat::Single => vec!["relative_path", "intensity"],
		LegacyFormat::Double => vec!["relative_path", "intensity", "external_intensity"],
	};

	writer
		.write_record(&headers)
		.map_err(|why| CSVBuilderError::new(format!("failed to write csv headers: {why}")))
}

/// Writes a single dataset entry as a row in the CSV file.
fn write_record(writer: &mut _FileWriter, entry: &LegacyEntry) -> Result<()> {
	let mut record = vec![entry.relative_path.clone()];

	record.push(entry.intensity.clone());

	if let Some(ext_intensity) = &entry.external_intensity {
		record.push(ext_intensity.clone());
	}

	writer
		.write_record(&record)
		.map_err(|why| CSVBuilderError::new(format!("failed to write csv record: {why}")))
}

/// Constructs a legacy entry by parsing the filename according to the detected format.
fn create_legacy_entry(
	root: &Path,
	entry: &DirEntry,
	format: &LegacyFormat,
) -> Result<LegacyEntry> {
	let relative_path = common::get_relative_path(root, entry.path())?;
	let segments = get_segments(entry)?;

	match format {
		LegacyFormat::Single => {
			if segments.len() != 2 {
				return Err(CSVBuilderError::new(
					"inconsistent dataset formatting detected",
				));
			}
			Ok(LegacyEntry {
				relative_path,
				external_intensity: None,
				intensity: segments[1].to_string(),
			})
		},
		LegacyFormat::Double => {
			if segments.len() != 3 {
				return Err(CSVBuilderError::new(
					"inconsistent dataset formatting detected",
				));
			}
			Ok(LegacyEntry {
				relative_path,
				external_intensity: Some(segments[1].to_string()),
				intensity: segments[2].to_string(),
			})
		},
	}
}
