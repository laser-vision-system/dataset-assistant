//! # Settings
//!
//! Compile-time constants.

pub const TEXTURE_WIDTH: usize = 640;
pub const TEXTURE_HEIGHT: usize = 480;
// Doesn't really have an effect because of text entry width.
pub const CONTROLS_WIDTH_MIN: f32 = 300.0;
pub const CONTROLS_WIDTH_MAX: f32 = 375.0;
pub const WINDOW_WIDTH: f32 = TEXTURE_WIDTH as f32 + CONTROLS_WIDTH_MAX;
pub const WINDOW_HEIGHT: f32 = TEXTURE_HEIGHT as f32;

pub const HEADING_FONT_SIZE: f32 = 18.0;
pub const SUBHEADING_FONT_SIZE: f32 = 14.0;

pub const HEADING_FONT_SPACING: f32 = 1.0;
pub const SUBHEADING_FONT_SPACING: f32 = 0.8;

pub const PREHEADING_SPACE: f32 = 5.0;

pub const DEFAULT_PATH_LABEL: &str = "No folder selected";
pub const DEFAULT_SUBSET_LABEL: &str = "Subset: ../..";
pub const DIRECTORY_SELECTED_LABEL: &str = "Root";
pub const NON_EMPTY_DIRECTORY_SELECTED_LABEL: &str = "Root (not empty)";
pub const FAILED_DIRECTORY_LABEL: &str = "Failed to select directory";

pub const DEFAULT_CAMERA_LABEL: &str = "None";

pub const FILE_NAME_BASE: &str = "capture";
/// Lossless format and quick to save.
pub const IMAGE_EXTENSION: &str = "png";
/// The accompanying file with metadata.
pub const META_EXTENSION: &str = "txt";
