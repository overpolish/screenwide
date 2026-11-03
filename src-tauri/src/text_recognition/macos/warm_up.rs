// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Vision's text networks run on the Neural Engine, which macOS compiles
//! them for once per build of the app that uses them, out of process and
//! about ten seconds each, while the read that needs one waits. Which ones a
//! read needs depends on the scripts it guesses at, not only those on the
//! page: a busy Latin page can load the Devanagari and Arabic readers too, and
//! wait half a minute for them. One read here, of a line in each script
//! Vision has a reader of its own for, while nobody is waiting, has every one
//! compiled before Text Recognition is used.

use std::sync::atomic::{AtomicBool, Ordering};

use screenwide_compositor::platform::type_device::TypeDevice;
use screenwide_text::engine::face::TypeFace;

/// One line in each script Vision reads with a network of its own: Latin
/// (which also covers Cyrillic), Devanagari, Arabic, Chinese (which also
/// covers Japanese), Korean and Thai, each large enough to be read, which is
/// what loads its reader.
const LINES: [&str; 6] = [
  "Screenwide 0123",
  "पाठ पहचान",
  "التعرف على النص",
  "文字识别",
  "텍스트 인식",
  "การรู้จำข้อความ",
];
const SIZE: f64 = 32.0;
const LINE_HEIGHT: f64 = 56.0;
const CELL: (u32, u32) = (360, LINES.len() as u32 * LINE_HEIGHT as u32 + 16);

/// Warms Vision once per process on a thread of its own, at the utility class:
/// nobody waits on it, and a read someone does wait on outranks it.
pub(in crate::text_recognition) fn start() {
  static STARTED: AtomicBool = AtomicBool::new(false);
  if STARTED.swap(true, Ordering::AcqRel) {
    return;
  }
  let spawned = std::thread::Builder::new()
    .name("text-recognition-warm-up".to_owned())
    .spawn(|| {
      let _urgency = crate::user_work::ThreadUrgency::ahead();
      if let Err(error) = read_once() {
        eprintln!("Could not prepare Text Recognition: {error}");
      }
    });
  if let Err(error) = spawned {
    eprintln!("Could not start preparing Text Recognition: {error}");
  }
}

fn read_once() -> Result<(), String> {
  let device = TypeDevice::in_face(TypeFace::new(None, 400, false), SIZE)?;
  let lines = LINES
    .iter()
    .enumerate()
    .map(|(index, line)| ((8.0, 12.0 + index as f64 * LINE_HEIGHT), *line))
    .collect::<Vec<_>>();
  let coverage = device.draw(CELL, &lines)?;
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
