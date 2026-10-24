//! Program entrypoint.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // Hide console window on Windows in release.

fn main() -> eframe::Result {
	dataset_assistant::run()
}
