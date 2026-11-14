use std::sync::Arc;

use eframe::egui;
use image::RgbImage;

use crate::settings::{
	TEXTURE_HEIGHT,
	TEXTURE_WIDTH,
};

/// Functionality for holding an image texture from a camera.
#[derive(Clone)]
pub struct Capture {
	pub texture: egui::TextureHandle,
	pub image: Option<Arc<egui::ColorImage>>,
}

impl Capture {
	/// Create a new capture texture.
	pub fn new(ctx: &egui::Context) -> Self {
		let texture = ctx.load_texture(
			"capture",
			egui::ImageData::Color(Arc::new(egui::ColorImage::new(
				[TEXTURE_WIDTH, TEXTURE_HEIGHT],
				vec![
					egui::Color32::TRANSPARENT;
					TEXTURE_WIDTH * TEXTURE_HEIGHT
				],
			))),
			egui::TextureOptions::default(),
		);
		Self {
			texture,
			image: None,
		}
	}

	/// Store an RGB image in the texture.
	pub fn store_image(&mut self, image: &RgbImage) {
		#[cfg(debug_assertions)]
		let (img_width, img_height) = image.dimensions();
		#[cfg(debug_assertions)]
		assert_dimensions(img_width as usize, img_height as usize);

		let image_arc = Arc::new(egui::ColorImage::from_rgb(
			[TEXTURE_WIDTH, TEXTURE_HEIGHT],
			image,
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
}

#[cfg(debug_assertions)]
fn assert_dimensions(img_width: usize, img_height: usize) {
	debug_assert_eq!(
		(img_width, img_height),
		(TEXTURE_WIDTH, TEXTURE_HEIGHT),
		"Expected dimentions: {TEXTURE_WIDTH}x{TEXTURE_HEIGHT}\nReceived dimensions: {img_width}x{img_height}"
	);
}
