// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Vision's text models load on their first read: the first ever on a Mac
//! can take tens of seconds while macOS prepares them, and the first in each
//! process a little longer than the rest. One read of a line set here, while
//! nobody is waiting, takes that cost before Text Recognition is used.

use std::sync::atomic::{AtomicBool, Ordering};

use screenwide_compositor::platform::type_device::TypeDevice;
use screenwide_text::engine::face::TypeFace;

/// The line read, and the cell it is set in, large enough for Vision to read
/// it, which is what loads the recogniser as well as the detector.
const LINE: &str = "Screenwide 0123";
const CELL: (u32, u32) = (320, 64);

/// Warms Vision once per process on a thread of its own.
pub(in crate::text_recognition) fn start() {
  static STARTED: AtomicBool = AtomicBool::new(false);
  if STARTED.swap(true, Ordering::AcqRel) {
    return;
  }
  let spawned = std::thread::Builder::new()
    .name("text-recognition-warm-up".to_owned())
    .spawn(|| {
      if let Err(error) = read_once() {
        eprintln!("Could not prepare Text Recognition: {error}");
      }
    });
  if let Err(error) = spawned {
    eprintln!("Could not start preparing Text Recognition: {error}");
  }
}

fn read_once() -> Result<(), String> {
  let device = TypeDevice::in_face(TypeFace::new(None, 400, false), 32.0)?;
  let coverage = device.draw(CELL, &[((8.0, 12.0), LINE)])?;
  // Black text on white, as most text Vision is asked to read.
  let rgba = coverage
    .iter()
    .flat_map(|coverage| {
      let ink = 255 - *coverage;
      [ink, ink, ink, 255]
    })
    .collect::<Vec<_>>();
  super::platform::recognize(&rgba, CELL.0, CELL.1).map(|_| ())
}
