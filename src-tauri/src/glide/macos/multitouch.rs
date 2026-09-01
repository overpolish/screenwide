// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{ffi::c_void, sync::Mutex, time::Duration};

use core_foundation::{
  array::{CFArrayGetCount, CFArrayGetValueAtIndex, CFArrayRef},
  runloop::{kCFRunLoopDefaultMode, CFRunLoop},
};
use core_graphics::{
  event::CGEvent,
  event_source::{CGEventSource, CGEventSourceStateID},
  geometry::CGPoint,
};
use tauri::AppHandle;

use super::session;

#[path = "multitouch/recognizer.rs"]
mod recognizer;

use recognizer::TapRecognizer;

const POLL_INTERVAL: Duration = Duration::from_millis(16);

#[repr(C)]
#[derive(Clone, Copy)]
struct MtPoint {
  x: f32,
  y: f32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct MtReadout {
  position: MtPoint,
  velocity: MtPoint,
}

/// The reverse-engineered contact record MultitouchSupport hands the frame
/// callback. Only `normalized.position` is read; the rest of the layout exists
/// to get the stride right, so a drift in the tail cannot corrupt the logic.
#[repr(C)]
#[derive(Clone, Copy)]
#[allow(dead_code)]
struct MtTouch {
  frame: i32,
  timestamp: f64,
  identifier: i32,
  state: i32,
  unknown_a: i32,
  unknown_b: i32,
  normalized: MtReadout,
  size: f32,
  unknown_c: i32,
  angle: f32,
  major_axis: f32,
  minor_axis: f32,
  absolute: MtReadout,
  unknown_d: [i32; 2],
  unknown_e: f32,
}

type MtContactCallback = extern "C" fn(
  device: *mut c_void,
  touches: *const MtTouch,
  num_touches: i32,
  timestamp: f64,
  frame: i32,
) -> i32;

extern "C" {
  fn MTDeviceCreateList() -> CFArrayRef;
  fn MTRegisterContactFrameCallback(device: *mut c_void, callback: MtContactCallback);
  fn MTDeviceStart(device: *mut c_void, run_mode: i32) -> i32;
}

/// The app handle the frame callback centers windows through. The callback is a
/// bare C function pointer, so the handle cannot ride along with it.
static APP: std::sync::OnceLock<AppHandle> = std::sync::OnceLock::new();
/// One recogniser per device, keyed by the device pointer, so two trackpads
/// cannot interleave their episodes into one state machine.
static RECOGNIZERS: Mutex<Vec<(usize, TapRecognizer)>> = Mutex::new(Vec::new());

pub(super) fn start(app: &AppHandle) {
  let _ = APP.set(app.clone());
  if let Err(error) = std::thread::Builder::new()
    .name("glide-multitouch".to_owned())
    .spawn(run)
  {
    eprintln!("Could not start Glide tap monitoring: {error}");
  }
}

/// Registers every trackpad with the frame callback and then services the run
/// loop the devices deliver on. The thread lives as long as the app.
fn run() {
  for device in devices() {
    // SAFETY: the device came from the list MultitouchSupport created and is
    // kept alive for the process lifetime, and the callback is a static fn.
    unsafe {
      MTRegisterContactFrameCallback(device, contact_frame);
      MTDeviceStart(device, 0);
    }
  }
  loop {
    // SAFETY: Core Foundation owns this process-global constant.
    unsafe {
      CFRunLoop::run_in_mode(kCFRunLoopDefaultMode, POLL_INTERVAL, false);
    }
  }
}

/// The multitouch devices attached at launch. The list is deliberately never
/// released: the device references it holds stay registered for the process
/// lifetime, and no device is added after this point.
fn devices() -> Vec<*mut c_void> {
  // SAFETY: `MTDeviceCreateList` returns a created array of device references,
  // or null when the framework finds no multitouch hardware.
  unsafe {
    let list = MTDeviceCreateList();
    if list.is_null() {
      return Vec::new();
    }
    (0..CFArrayGetCount(list))
      .map(|index| CFArrayGetValueAtIndex(list, index).cast_mut())
      .collect()
  }
}

/// The raw frame callback. It does no more than derive the recogniser's three
/// inputs from the frame, so the trackpad's delivery thread stays free.
extern "C" fn contact_frame(
  device: *mut c_void,
  touches: *const MtTouch,
  num_touches: i32,
  timestamp: f64,
  _frame: i32,
) -> i32 {
  let count = num_touches.max(0) as usize;
  let centroid = centroid(touches, count);
  let tapped = RECOGNIZERS.lock().ok().and_then(|mut recognizers| {
    recognizer_for(&mut recognizers, device as usize).update(count, centroid, timestamp)
  });
  if tapped.is_some() {
    if let (Some(app), Some(point)) = (APP.get(), cursor_position()) {
      session::register_tap(app, point);
    }
  }
  0
}

fn recognizer_for(
  recognizers: &mut Vec<(usize, TapRecognizer)>,
  device: usize,
) -> &mut TapRecognizer {
  let index = match recognizers.iter().position(|(key, _)| *key == device) {
    Some(index) => index,
    None => {
      recognizers.push((device, TapRecognizer::default()));
      recognizers.len() - 1
    }
  };
  &mut recognizers[index].1
}

/// The average of the frame's contacts, in normalised 0..1 trackpad units.
fn centroid(touches: *const MtTouch, count: usize) -> Option<(f32, f32)> {
  if touches.is_null() || count == 0 {
    return None;
  }
  // SAFETY: the callback owns `count` contiguous contacts for its duration.
  let touches = unsafe { std::slice::from_raw_parts(touches, count) };
  let (x, y) = touches.iter().fold((0.0, 0.0), |(x, y), touch| {
    (
      x + touch.normalized.position.x,
      y + touch.normalized.position.y,
    )
  });
  let count = count as f32;
  Some((x / count, y / count))
}

/// Where the cursor is right now. A tap carries no location of its own, so the
/// pointer decides which window the gesture landed on.
fn cursor_position() -> Option<CGPoint> {
  let source = CGEventSource::new(CGEventSourceStateID::CombinedSessionState).ok()?;
  CGEvent::new(source).ok().map(|event| event.location())
}
