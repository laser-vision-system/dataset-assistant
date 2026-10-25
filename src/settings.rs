#![expect(
	clippy::cast_precision_loss,
	reason = "The window dimensions will never get big enough to matter"
)]

//! # Settings
//!
//! Compile-time constants.

pub const TEXTURE_WIDTH: usize = 640;
pub const TEXTURE_HEIGHT: usize = 480;

pub const DISPLAY_MIN_WIDTH: f32 = TEXTURE_WIDTH as f32;
pub const DISPLAY_MIN_HEIGHT: f32 = TEXTURE_HEIGHT as f32;

pub const CONTROLS_MIN_WIDTH: f32 = 300.0;
pub const CONTROLS_MAX_WIDTH: f32 = 550.0;

pub const EXPANDED_MIN_WIDTH: f32 = DISPLAY_MIN_WIDTH + CONTROLS_MAX_WIDTH;
pub const EXPANDED_MIN_HEIGHT: f32 = DISPLAY_MIN_HEIGHT;

pub const EXPANDED_MAX_WIDTH: f32 = 0.0;
pub const EXPANDED_MAX_HEIGHT: f32 = 0.0;

pub const DEFAULT_WIDTH: f32 = CONTROLS_MAX_WIDTH;
pub const DEFAULT_HEIGHT: f32 = DISPLAY_MIN_HEIGHT;

pub const DEFAULT_MIN_WIDTH: f32 = CONTROLS_MIN_WIDTH;
pub const DEFAULT_MIN_HEIGHT: f32 = DISPLAY_MIN_HEIGHT;

pub const DEFAULT_MAX_WIDTH: f32 = CONTROLS_MAX_WIDTH;
pub const DEFAULT_MAX_HEIGHT: f32 = 0.0;

pub const FRAME_INNER_MARGIN: f32 = 2.0;
pub const FRAME_STROKE_WIDTH: f32 = 1.0;
pub const FRAME_STROKE_GREY: u8 = 50;

pub const HEADING_FONT_SIZE: f32 = 18.0;
pub const SUBHEADING_FONT_SIZE: f32 = 14.0;

pub const HEADING_FONT_SPACING: f32 = 1.0;
pub const SUBHEADING_FONT_SPACING: f32 = 0.8;

pub const PREHEADING_SPACE: f32 = 2.0;

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
