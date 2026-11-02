// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! One open preview: its place in the window, and the thread drawing it.

use std::sync::mpsc::{self, Sender};
use std::thread::JoinHandle;

use screenwide_gpu::surface::Surface;
use screenwide_imaging::CapturedImage;

use crate::host::{Host, HostWindow, Stacking};
use crate::placement::PreviewPlacement;
use crate::scene::driver::SceneDriver;
use crate::scene::{PreviewScene, SceneRenderer};
use crate::worker::{self, Message, Renderer};

pub struct Preview {
  host: Option<Host>,
  messages: Sender<Message>,
  worker: Option<JoinHandle<()>>,
  placed: Option<PreviewPlacement>,
}

impl Preview {
  /// Opens a preview in `window`, hidden until placed, drawn on a thread of
  /// its own by what `renderer` makes of its surface there.
  pub fn open<F>(window: &HostWindow, stacking: Stacking, renderer: F) -> Result<Self, String>
  where
    F: FnOnce(Surface) -> Result<Box<dyn Renderer>, String> + Send + 'static,
  {
    let (host, surface) = Host::open(window, stacking)?;
    let (messages, receiver) = mpsc::channel();
    let worker = std::thread::Builder::new()
      .name("settings-preview".to_owned())
      .spawn(move || match renderer(surface) {
        Ok(renderer) => worker::run(renderer, &receiver),
        Err(error) => eprintln!("A settings preview could not start: {error}"),
      })
      .map_err(|error| error.to_string())?;
    Ok(Self {
      host: Some(host),
      messages,
      worker: Some(worker),
      placed: None,
    })
  }

  /// Opens a recording `scene` of `pictures`, the first `points` in size,
  /// over the page in `window`.
  pub fn open_recording(
    window: &HostWindow,
    scene: PreviewScene,
    pictures: Vec<CapturedImage>,
    points: (f64, f64),
  ) -> Result<Self, String> {
    let compositor = screenwide_compositor::compositor::Compositor::new(screenwide_gpu::shared()?)?;
    let renderer = SceneRenderer::new(compositor, scene, pictures, points)?;
    Self::open(window, Stacking::Above, move |surface| {
      Ok(Box::new(SceneDriver::new(renderer, surface)?) as Box<dyn Renderer>)
    })
  }

  /// Moves, clips, rounds, shows or hides the preview. The thread drawing
  /// it hears of a new size on screen, and of being shown or hidden.
  pub fn place(&mut self, placement: PreviewPlacement) {
    if let Some(host) = &self.host {
      host.place(placement);
    }
    let previous = self.placed.replace(placement);
    let pixels = placement.picture_pixels();
    if previous.map(|previous| previous.picture_pixels()) != Some(pixels) {
      self.send(Message::Resize(pixels));
    }
    if previous.map(|previous| previous.visible) != Some(placement.visible) {
      self.send(Message::Visible(placement.visible));
    }
  }

  pub fn send(&self, message: Message) {
    // A thread that has stopped has nothing left to draw.
    let _ = self.messages.send(message);
  }
}

impl Drop for Preview {
  /// The thread draws into the host's layer, so it stops before the layer
  /// goes.
  fn drop(&mut self) {
    self.send(Message::Stop);
    if let Some(worker) = self.worker.take() {
      let _ = worker.join();
    }
    drop(self.host.take());
  }
}
