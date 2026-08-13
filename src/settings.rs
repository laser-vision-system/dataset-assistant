//! # Settings
//!
//! Compile-time constants.

pub const TEXTURE_WIDTH: usize = 640;
pub const TEXTURE_HEIGHT: usize = 480;
pub const CONTROLS_WIDTH: f32 = 350.0;
pub const WINDOW_WIDTH: f32 = TEXTURE_WIDTH as f32 + CONTROLS_WIDTH;
pub const WINDOW_HEIGHT: f32 = TEXTURE_HEIGHT as f32;
pub const TITLE_FONT_SIZE: f32 = 18.0;

pub const DEFAULT_CAMERA_LABEL: &str = "No camera";
pub const DEFAULT_PATH_LABEL: &str = "No folder selected";
pub const DEFAULT_SUBSET_LABEL: &str = "../..";

pub const FILE_NAME_BASE: &str = "capture";
/// Lossless format and quick to save.
pub const IMAGE_EXTENSION: &str = "png";
/// The accompanying file with metadata.
pub const META_EXTENSION: &str = "txt";
