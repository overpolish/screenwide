// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The matte's frames cleaned up on several threads at once and written in
//! turn. A frame's cleanup takes nothing from the frame before it, so frames
//! cleaned side by side make the same matte; at about 10 ms a 960 by 540
//! frame, one thread keeps up with the model on the CPU but not on the GPU.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{sync_channel, Receiver};
use std::sync::{Mutex, PoisonError};

use super::cleanup::Cleanup;
use super::encode::Encoder;

/// A frame of RVM's matte, numbered in turn from 0, and the 8-bit RGB camera
/// frame it was made from.
pub(super) type Matted = (u64, Vec<u8>, Vec<u8>);

/// Cleans up the frames `frames` brings, `size` pixels each, on `cleaners`
/// threads, and writes them to `encoder` in turn, counting each in `done`.
pub(super) fn clean_in_order(
  frames: Receiver<Matted>,
  mut encoder: Encoder,
  size: (usize, usize),
  cleaners: usize,
  done: &AtomicU64,
) -> Result<(), String> {
  let cleaners = cleaners.max(1);
  let frames = Mutex::new(frames);
  let (send, cleaned) = sync_channel::<(u64, Vec<u8>)>(cleaners * 2);
  std::thread::scope(|scope| {
    for _ in 0..cleaners {
      let (send, frames) = (send.clone(), &frames);
      scope.spawn(move || {
        let mut cleanup = Cleanup::new(size);
        loop {
          // Held only while waiting for the next frame.
          let next = frames.lock().unwrap_or_else(PoisonError::into_inner).recv();
          let Ok((index, mut alpha, rgb)) = next else {
            break;
          };
          cleanup.apply(&mut alpha, &rgb);
          if send.send((index, alpha)).is_err() {
            break;
          }
        }
      });
    }
    drop(send);
    // Frames cleaned ahead of the next one to write wait here, at most a
    // few: each cleaner holds one frame at a time.
    let mut waiting = BTreeMap::new();
    let mut next = 0;
    for (index, alpha) in cleaned {
      waiting.insert(index, alpha);
      while let Some(alpha) = waiting.remove(&next) {
        encoder.write(&alpha)?;
        done.fetch_add(1, Ordering::Relaxed);
        next += 1;
      }
    }
    Ok::<_, String>(())
  })?;
  encoder.finish()
}
