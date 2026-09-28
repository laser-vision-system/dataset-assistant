use std::assert_matches;

use super::*;

#[test]
fn available_path() {
	let path = Path::new(
		"./long/path/guaranteed/to/be/available/a/a/a/a/a/a/a/a/a/a/a/a/",
	);
	assert_matches!(is_path_available(path), Ok(()))
}

#[test]
fn unavailable_path_file() {
	let path = Path::new("./examples/dataset_assistant/saving/dataset.rs");
	assert_matches!(is_path_available(path), Err(why) if why.kind() == io::ErrorKind::NotADirectory)
}

#[test]
fn unavailable_path_not_empty() {
	let path = Path::new("./examples/dataset_assistant/");
	assert_matches!(is_path_available(path), Err(why) if why.kind() == io::ErrorKind::DirectoryNotEmpty)
}

#[test]
fn dataset_try_new() {
	let path = Path::new("./temp/dataset");
	let ds = Dataset::try_from(path.to_path_buf());
	assert_eq!(ds.unwrap().root, path.to_path_buf());
}

// Other method testing would be better suited for integration tests, which
// I'm not bothering with for this.
