//! Camera ownership with threaded polling.

use std::{
	sync::{
		Arc,
		Mutex,
		atomic::{
			AtomicBool,
			Ordering,
		},
		mpsc::{
			self,
			Receiver,
			TrySendError,
		},
	},
	thread,
	time::Duration,
};

use edge_cameras::camera_device::{
	CameraDevice,
	CameraLike as _,
};
use edge_core::image::Image;
use eframe::egui;

const SLEEP_DURATION: Duration = Duration::from_millis(20);

/// Camera handle with non-blocking frame delivery for the UI.
pub struct CameraSession {
	camera: Arc<Mutex<CameraDevice>>,
	live_feed: Arc<AtomicBool>,
	requested_capture: Arc<AtomicBool>,
	stop: Arc<AtomicBool>,
	frames: Receiver<Image>,
}

impl CameraSession {
	/// Start a frame-acquisition worker for an opened camera.
	///
	/// The worker only acquires the camera mutex opportunistically, allowing
	/// the save procedure to retain exclusive camera access for its full
	/// duration.
	///
	/// # Errors
	///
	/// Returns an error if the acquisition thread cannot be started.
	pub fn new(
		camera: Arc<Mutex<CameraDevice>>,
		ctx: egui::Context,
	) -> std::io::Result<Self> {
		let (frame_sender, frames) = mpsc::sync_channel(1);
		let live_feed = Arc::new(AtomicBool::new(false));
		let requested_capture = Arc::new(AtomicBool::new(false));
		let stop = Arc::new(AtomicBool::new(false));

		let worker_live_feed = Arc::clone(&live_feed);
		let worker_capture = Arc::clone(&requested_capture);
		let worker_stop = Arc::clone(&stop);
		let worker_camera = Arc::clone(&camera);

		thread::Builder::new()
			.name(String::from("camera-preview"))
			.spawn(move || {
				while !worker_stop.load(Ordering::Relaxed) {
					let live = worker_live_feed.load(Ordering::Relaxed);
					let requested =
						worker_capture.swap(false, Ordering::Relaxed);
					if !live && !requested {
						thread::sleep(SLEEP_DURATION);
						continue;
					}

					if let Ok(mut camera) = worker_camera.try_lock() {
						if let Ok(buffer) = camera.get_frame_buffer()
							&& let Ok(image) = buffer.try_decode_to_image()
						{
							ctx.request_repaint();
							match frame_sender.try_send(image) {
								Ok(()) | Err(TrySendError::Full(_)) => {},
								Err(TrySendError::Disconnected(_)) => break,
							}
						}
					} else {
						if requested {
							worker_capture.store(true, Ordering::Relaxed);
						}
						thread::sleep(SLEEP_DURATION);
					}
				}
			})?;

		Ok(Self {
			camera,
			live_feed,
			requested_capture,
			stop,
			frames,
		})
	}

	/// Return a shared camera handle for control updates and saving.
	pub fn camera(&self) -> Arc<Mutex<CameraDevice>> {
		Arc::clone(&self.camera)
	}

	/// Set whether the worker continuously acquires preview frames.
	pub fn set_live_feed(&self, enabled: bool) {
		self.live_feed.store(enabled, Ordering::Relaxed);
	}

	/// Ask the worker for a single frame.
	pub fn request_capture(&self) {
		self.requested_capture.store(true, Ordering::Relaxed);
	}

	/// Take the newest available frame.
	pub fn poll_frame(&self) -> Option<Image> {
		let mut latest = None;
		while let Ok(image) = self.frames.try_recv() {
			latest = Some(image);
		}
		latest
	}
}

impl Drop for CameraSession {
	fn drop(&mut self) {
		self.stop.store(true, Ordering::Relaxed);
	}
}
