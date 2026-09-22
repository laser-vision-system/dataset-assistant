//! A CSV builder of the standard format.

use std::{
	fs,
	path::{Path, PathBuf},
	thread,
};

use walkdir::{DirEntry, WalkDir};

use super::{
	_FileWriter, _JoinHandle, CSVBuilderError, CSVBuilderHandle, Result, common,
};

const OUTPUT_NAME: &str = "index.csv";

#[derive(Debug)]
pub struct Builder {
	handle: Option<_JoinHandle>,
}

impl Builder {
	pub fn new(path: PathBuf) -> Self {
		let handle = thread::Builder::new()
			.name("csv_builder".into())
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

type _MetadataList = Vec<MetadataField>;

/// Represents a parsed metadata key-value pair.
#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct MetadataField {
	key: String,
	value: u16,
}

/// Contains all extracted information for a single image dataset entry.
#[derive(Clone, Debug, PartialEq, PartialOrd)]
struct DatasetEntry {
	relative_path: String,
	position: String,
	intensity: f32,
	metadata: _MetadataList,
}

/// Main execution pipeline for the CSV builder thread.
fn run(root_path: PathBuf) -> Result<()> {
	let mut entry_stream = WalkDir::new(&root_path)
		.min_depth(1)
		.into_iter()
		.filter_entry(common::is_not_git_dir)
		.filter_map(|e| e.ok())
		.filter(common::is_image_file)
		.map(|entry| create_dataset_entry(&root_path, &entry));

	let csv_path = root_path.join(OUTPUT_NAME);
	let mut writer = csv::Writer::from_path(csv_path).map_err(|why| {
		CSVBuilderError::new(format!("failed to create csv writer: {why}"))
	})?;

	write_stream(&mut writer, &mut entry_stream)?;

	writer.flush().map_err(|why| {
		CSVBuilderError::new(format!("failed to flush csv: {why}"))
	})
}

/// Constructs a single dataset entry by parsing the path and reading metadata.
fn create_dataset_entry(
	root: &Path,
	image_entry: &DirEntry,
) -> Result<DatasetEntry> {
	let image_path = image_entry.path();
	let relative_path = common::get_relative_path(root, image_path)?;

	let intensity_dir = get_parent_directory(image_path)?;
	let position_dir = get_parent_directory(intensity_dir)?;

	let intensity_name = common::get_file_name(intensity_dir)?;
	let intensity = parse_intensity(&intensity_name)?;
	let position = common::get_file_name(position_dir)?;

	let metadata_path = image_path.with_extension("txt");
	let metadata = read_metadata(&metadata_path)?;

	Ok(DatasetEntry {
		relative_path,
		position,
		intensity,
		metadata,
	})
}

/// Extracts the parent directory path from a given path.
fn get_parent_directory(path: &Path) -> Result<&Path> {
	path.parent()
		.ok_or_else(|| CSVBuilderError::new("missing parent directory in path"))
}

/// Parses a directory name into an f32 intensity value.
fn parse_intensity(name: &str) -> Result<f32> {
	name.parse::<f32>().map_err(|why| {
		CSVBuilderError::new(format!("failed to parse intensity: {why}"))
	})
}

/// Reads and parses a metadata file into a list of fields.
fn read_metadata(path: &Path) -> Result<_MetadataList> {
	let content = fs::read_to_string(path).map_err(|why| {
		CSVBuilderError::new(format!("failed to read metadata file: {why}"))
	})?;

	content
		.lines()
		.filter(|line| !line.trim().is_empty())
		.map(parse_metadata_line)
		.collect()
}

/// Parses a single metadata line formatted as `key: value`.
fn parse_metadata_line(line: &str) -> Result<MetadataField> {
	let (key, value) = line
		.split_once(':')
		.ok_or_else(|| CSVBuilderError::new("invalid metadata line format"))?;

	let parsed_value = value.trim().parse::<u16>().map_err(|why| {
		CSVBuilderError::new(format!("failed to parse metadata value: {why}"))
	})?;

	Ok(MetadataField {
		key: key.trim().to_string(),
		value: parsed_value,
	})
}

/// Processes the iterator stream, writing headers and all subsequent records.
fn write_stream(
	writer: &mut _FileWriter,
	stream: &mut impl Iterator<Item = Result<DatasetEntry>>,
) -> Result<()> {
	// Build headers from the first result.
	let Some(first_result) = stream.next() else {
		return Ok(());
	};
	let first_entry = first_result?;
	write_headers(writer, &first_entry)?;
	write_record(writer, &first_entry)?;

	for result in stream {
		write_record(writer, &result?)?;
	}

	Ok(())
}

/// Writes the header row to the CSV file based on the first entry's metadata.
fn write_headers(
	writer: &mut _FileWriter,
	first_entry: &DatasetEntry,
) -> Result<()> {
	let mut headers = vec![
		"relative_path".to_string(),
		"position".to_string(),
		"intensity".to_string(),
	];

	for field in &first_entry.metadata {
		headers.push(field.key.clone());
	}

	writer.write_record(&headers).map_err(|why| {
		CSVBuilderError::new(format!("failed to write csv headers: {why}"))
	})
}

/// Writes a single dataset entry as a row in the CSV file.
fn write_record(writer: &mut _FileWriter, entry: &DatasetEntry) -> Result<()> {
	let mut record = vec![
		entry.relative_path.clone(),
		entry.position.clone(),
		entry.intensity.to_string(),
	];

	for field in &entry.metadata {
		record.push(field.value.to_string());
	}

	writer.write_record(&record).map_err(|why| {
		CSVBuilderError::new(format!("failed to write csv record: {why}"))
	})
}
