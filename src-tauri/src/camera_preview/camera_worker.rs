// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

pub(super) fn build_camera_preview(
  device_id: &str,
  width: u32,
  height: u32,
  fps: u32,
  #[cfg_attr(not(target_os = "windows"), allow(unused_variables))] pal: bool,
  channel: Channel,
) -> Result<CameraPreviewWorker, String> {
  let camera_info = query(ApiBackend::Auto)
    .map_err(|error| error.to_string())?
    .into_iter()
    .find(|camera| camera_id(camera) == device_id)
    .ok_or_else(|| "The selected camera is no longer available".to_owned())?;
  let camera_index = camera_info.index().clone();
  // AVFoundation already supplied this exact native mode during passive
  // enumeration. Constructing a Nokhwa Camera here just to enumerate it again
  // opens the device twice in immediate succession and can leave Continuity
  // cameras busy before the preview worker starts. Reading `formats()` off a
  // cidre `av::CaptureDevice` is passive and does not open the device.
  //
  // Nokhwa's AVFoundation backend only accepts an fps that equals one of the
  // mode's frame rate range maximums, so a PAL request (25/50) against a 1-30
  // range is rejected outright. Open at a rate it accepts and pin the real
  // frame duration with cidre once the stream is running.
  #[cfg(target_os = "macos")]
  let camera_name = camera_info.human_name();
  #[cfg(target_os = "macos")]
  let device_id = device_id.to_owned();
  #[cfg(target_os = "macos")]
  let open_fps = match camera_frame_rate::resolve_device(&device_id, &camera_name) {
    Ok(device) => camera_frame_rate::nokhwa_frame_rate(&device, width, height, fps),
    Err(_) => fps,
  };
  #[cfg(target_os = "macos")]
  let format = CameraFormat::new(Resolution::new(width, height), FrameFormat::YUYV, open_fps);
  #[cfg(not(target_os = "macos"))]
  let format = resolve_exact_camera_format(&camera_index, width, height, fps)?;
  // Windows cannot reach a PAL cadence through Media Foundation, so anti-flicker
  // is the camera's own power line frequency control instead. Applied before
  // the device opens; a camera without the control still previews.
  #[cfg(target_os = "windows")]
  if let Err(error) = crate::camera_power_line::apply_power_line_frequency(device_id, pal) {
    eprintln!("The camera's power line frequency was not set: {error}");
  }
  let worker_device_id = device_id.to_owned();
  let cancelled = Arc::new(AtomicBool::new(false));
  let owner_cancelled = Arc::clone(&cancelled);
  let callback_cancelled = Arc::clone(&cancelled);
  let delivery = PreviewDelivery::spawn(channel)?;
  let preview_frames = delivery.sender();
  let (started_tx, started) = mpsc::channel();
  let thread = std::thread::Builder::new()
    .name("camera-preview".to_owned())
    .spawn(move || {
      let requested = RequestedFormat::new::<RgbAFormat>(RequestedFormatType::Exact(format));
      let mut camera = match CallbackCamera::new(camera_index, requested, move |frame| {
        if callback_cancelled.load(Ordering::Acquire) {
          return;
        }
        let _ = preview_frames.try_send(frame);
      }) {
        Ok(camera) => camera,
        Err(error) => {
          let _ = started_tx.send(Err(error.to_string()));
          return;
        }
      };
      // `arc::R<av::CaptureDevice>` is not `Send`, so the device is resolved
      // here rather than moved in; the lookup is a passive registry hit.
      // Always pinned, even when nokhwa opened at `fps` itself: the device
      // keeps its last frame duration across sessions, so a 30 fps preview
      // following a 25 fps one must state its rate explicitly. A failed pin
      // leaves the preview running at `open_fps`, which is still a usable
      // picture, so it is reported and not treated as a start failure.
      #[cfg(target_os = "macos")]
      let pin = || {
        let pinned =
          camera_frame_rate::resolve_device(&device_id, &camera_name).and_then(|mut device| {
            camera_frame_rate::pin_frame_rate(&mut device, width, height, fps)
          });
        if let Err(error) = pinned {
          eprintln!("The camera preview stayed at {open_fps} fps instead of {fps} fps: {error}");
        }
      };
      // Pinned on both sides of the session start. A Continuity Camera locks
      // its rate in when the session starts and will only go lower afterwards,
      // so the device must already be at `fps` before nokhwa starts running;
      // the second pin wins back anything nokhwa re-applied during its own
      // configuration (it resets the duration to the range maximum).
      #[cfg(target_os = "macos")]
      pin();
      if let Err(error) = camera.open_stream() {
        let _ = started_tx.send(Err(error.to_string()));
        return;
      }
      #[cfg(target_os = "macos")]
      pin();
      if started_tx.send(Ok(())).is_err() {
        return;
      }

      while !owner_cancelled.load(Ordering::Acquire) {
        std::thread::sleep(Duration::from_millis(5));
      }
      // CallbackCamera is closed by the same worker that created and owns it.
      drop(camera);
    })
    .map_err(|error| error.to_string())?;

  let worker = CameraPreviewWorker {
    cancelled,
    delivery: Some(delivery),
    device_id: worker_device_id,
    fps,
    thread: Some(thread),
  };
  started
    .recv()
    .map_err(|_| "The camera preview worker stopped before starting".to_owned())??;
  Ok(worker)
}
