use std::{path::PathBuf, sync::Arc};

use eframe::egui;
use image::RgbImage;

use crate::settings::{TEXTURE_HEIGHT, TEXTURE_WIDTH};

/// Functionality for holding an image from a camera.
#[derive(Clone)]
pub struct Capture {
	pub texture: egui::TextureHandle,
	pub image: Option<Arc<egui::ColorImage>>,
}

impl Capture {
	pub fn new(ctx: &egui::Context) -> Self {
		let texture = ctx.load_texture(
			"capture",
			egui::ImageData::Color(Arc::new(egui::ColorImage::new(
				[TEXTURE_WIDTH, TEXTURE_HEIGHT],
				vec![egui::Color32::TRANSPARENT; TEXTURE_WIDTH * TEXTURE_HEIGHT],
			))),
			egui::TextureOptions::default(),
		);
		Self {
			texture,
			image: None,
		}
	}

	pub fn store_image(&mut self, image: RgbImage) {
		let image_arc = Arc::new(egui::ColorImage::from_rgb(
			[TEXTURE_WIDTH, TEXTURE_HEIGHT],
			&image,
		));
		self.image = Some(image_arc.clone());
		self.texture
			.set(image_arc.clone(), egui::TextureOptions::default());
	}

	pub fn display_image(&mut self, ui: &mut egui::Ui) {
		if self.image.is_none() {
			ui.centered_and_justified(|ui| {
				ui.spinner();
			});
		} else {
			ui.image((self.texture.id(), ui.available_size()));
		}
	}

	pub fn save_to_file(&mut self, path: &PathBuf) {
		let Some(capture) = self.image.as_ref() else {
			eprintln!("No image to save");
			return;
		};

		let res = image::save_buffer(
			path,
			capture.as_raw(),
			capture.width() as u32,
			capture.height() as u32,
			image::ColorType::Rgba8,
		);
		match res {
			Err(e) => eprintln!("Error saving capture {e}"),
			_ => (),
		};
	}
}
