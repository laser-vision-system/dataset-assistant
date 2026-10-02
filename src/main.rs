//! # Dataset Assistant
//!
//! Helper application for building datasets from real-world data. It uses the
//! T-cam Interface from the main codebase. This is a mutually beneficial
//! relationship that improves the development at both ends.
//!
//! ## Intended Functionality
//!
//! The intended functionality of this program is for the user to:
//!
//! - First set the camera up in the intended location and measure the laser
//!   intensity.
//! - Within this app, choose an output directory; it will likely be a
//!   sequentially numbered folder.
//! - Provide a named identifier for the camera position.
//! - Enter the current measured laser intensity.
//!
//! The system will then automatically capture images, adjusting the camera
//! settings through all available options (potentially provide user settings to
//! limit this).
//!
//! I would like for the UI to remain active during the automatic capture, to
//! display information about the current progress.
//!
//! The UI can display an image from the camera, which would be useful during
//! setup to verify the camera placement is correct. Image display during the
//! automatic collection would be desirable, but can be forgoed for performance
//! reasons.
//!
//! There may need to be a way to pick a specific camera to use.
//!
//! ## Output
//!
//! The dataset will be formatted as so:
//!
//! ```rs.ignore
//! dataset_root_folder/   <- The folder the user chooses.
//!  ├─ position1/   <- A named position of the camera.
//!  │  ├─ 1.21/   <- The measured laser intensity.
//!  │  │  ├─ image0.png
//!  │  │  ├─ image1.png
//!  │  │  ├─ image2.png
//!  │  │  ├─ image3.png
//!  │  │  └─ ...
//!  │  ├─ 1.87/
//!  │  │  ├─ image0.png
//!  │  │  ├─ image1.png
//!  │  │  ├─ image2.png
//!  │  │  ├─ image3.png
//!  │  │  └─ ...
//!  │  └─ ...
//!  ├─ position2/   <- A new camera position.
//!  │  ├─ 1.22/   <- Measured laser intensity might not be exact between positions.
//!  │  │  ├─ image0.png
//!  │  │  ├─ image1.png
//!  │  │  ├─ image2.png
//!  │  │  ├─ image3.png
//!  │  │  └─ ...
//!  │  ├─ 1.82/
//!  │  │  ├─ image0.png
//!  │  │  ├─ image1.png
//!  │  │  ├─ image2.png
//!  │  │  ├─ image3.png
//!  │  │  └─ ...
//!  │  └─ ...
//!  └─ ...
//! ```

#![expect(
	clippy::expect_used,
	reason = "This project is allowed to panic when unexpected things happen"
)]
#![expect(
	clippy::print_stderr,
	reason = "This project is allowed to print errors instead of panicking"
)]
#![expect(clippy::result_large_err, reason = "Issue present in external crate")]
#![cfg_attr(
	test,
	expect(clippy::unwrap_used, reason = "Tests are allowed to unwrap")
)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // Hide console window on Windows in release.

use eframe::{
	EventLoopBuilder, EventLoopBuilderHook, NativeOptions, Renderer, UserEvent,
	egui::ViewportBuilder,
};
#[cfg(target_os = "linux")]
use winit::platform::wayland::EventLoopBuilderExtWayland as _; // Bring into context.

mod application;
mod camera_controls;
mod capture;
mod csv_builder;
mod dataset;
mod entry;
mod runner;
mod saving;
mod settings;
mod state;
mod subset;
mod sweep;
mod user_interface;

use self::{
	application::DatasetAssistantApp,
	settings::{
		DEFAULT_HEIGHT, DEFAULT_MAX_HEIGHT, DEFAULT_MAX_WIDTH,
		DEFAULT_MIN_HEIGHT, DEFAULT_MIN_WIDTH, DEFAULT_WIDTH,
	},
};

fn main() -> eframe::Result {
	let event_loop_builder: Option<EventLoopBuilderHook> = cfg_select! {
		target_os = "linux" => {
			Some(Box::new(|builder: &mut EventLoopBuilder<UserEvent>| { builder.with_any_thread(true); }))
		},
		_ => {
			None
		}
	};

	let options = NativeOptions {
		event_loop_builder,
		renderer: Renderer::Glow,
		viewport: ViewportBuilder::default()
			.with_inner_size([DEFAULT_WIDTH, DEFAULT_HEIGHT])
			.with_min_inner_size([DEFAULT_MIN_WIDTH, DEFAULT_MIN_HEIGHT])
			.with_max_inner_size([DEFAULT_MAX_WIDTH, DEFAULT_MAX_HEIGHT]),
		..NativeOptions::default()
	};

	eframe::run_native(
		"eframe example",
		options,
		Box::new(|cc| {
			egui_extras::install_image_loaders(&cc.egui_ctx);
			Ok(Box::new(DatasetAssistantApp::new(&cc.egui_ctx)))
		}),
	)
}
