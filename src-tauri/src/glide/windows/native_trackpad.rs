// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Contact-driven Precision Touchpad input. HID report descriptors provide
//! the contact collections, so this works without Apple-specific packet
//! offsets; devices that do not expose them continue through wheel fallback.

use std::{
  cell::RefCell,
  collections::HashMap,
  sync::atomic::{AtomicBool, Ordering},
  time::{Duration, Instant},
};

use windows::Win32::UI::Input::{RAWHID, RAWINPUT, RAWINPUTHEADER};

use super::{begin_session, native_settings, session, InputKind, APP};

const FALLBACK_SUPPRESSION: Duration = Duration::from_millis(180);

#[path = "native_trackpad/hid.rs"]
mod hid;
#[path = "native_trackpad/pointer.rs"]
mod pointer;

use pointer::PointerEpisode;

thread_local! {
  static DEVICES: RefCell<HashMap<isize, Device>> = RefCell::new(HashMap::new());
  static LAST_NATIVE_CONTACT: RefCell<Option<Instant>> = const { RefCell::new(None) };
}
static POINTER_EPISODE: AtomicBool = AtomicBool::new(false);

struct Device {
  hid: hid::ContactParser,
  last_centroid: Option<(f64, f64)>,
  driving: bool,
  ignored: bool,
  pointer: PointerEpisode,
}

pub(super) fn handle_raw_input(header: &RAWINPUTHEADER, hid: &RAWHID, packet_bytes: usize) {
  let report_bytes = (hid.dwSizeHid as usize).saturating_mul(hid.dwCount as usize);
  let data_offset = std::mem::offset_of!(RAWINPUT, data) + std::mem::offset_of!(RAWHID, bRawData);
  let length = report_bytes.min(packet_bytes.saturating_sub(data_offset));
  if hid.dwSizeHid == 0 || length < hid.dwSizeHid as usize {
    return;
  }
  let reports = unsafe { std::slice::from_raw_parts(hid.bRawData.as_ptr(), length) };
  let key = header.hDevice.0 as isize;
  DEVICES.with_borrow_mut(|devices| {
    if let std::collections::hash_map::Entry::Vacant(entry) = devices.entry(key) {
      let Some(device) = Device::open(header.hDevice) else {
        return;
      };
      entry.insert(device);
    }
    let Some(device) = devices.get_mut(&key) else {
      return;
    };
    for report in reports.chunks_exact(hid.dwSizeHid as usize) {
      device.handle_report(report);
    }
    POINTER_EPISODE.store(
      devices.values().any(|device| device.pointer.active()),
      Ordering::Release,
    );
  });
}

pub(super) fn blocks_mouse_glide(mouse_modifier_down: bool) -> bool {
  mouse_glide_blocked(mouse_modifier_down, POINTER_EPISODE.load(Ordering::Acquire))
}

fn mouse_glide_blocked(mouse_modifier_down: bool, pointer_episode: bool) -> bool {
  mouse_modifier_down && pointer_episode
}

pub(super) fn suppresses_scroll_fallback() -> bool {
  LAST_NATIVE_CONTACT.with_borrow_mut(|last| extend_suppression(last, Instant::now()))
}

fn extend_suppression(last: &mut Option<Instant>, now: Instant) -> bool {
  let suppress = last.is_some_and(|last| now.duration_since(last) < FALLBACK_SUPPRESSION);
  if suppress {
    // Windows can keep synthesizing momentum wheel packets after both
    // contacts lift. Each packet extends the quiet window so no late tail
    // can open a second, phase-less fallback session and hide the cursor.
    *last = Some(now);
  }
  suppress
}

impl Device {
  fn open(handle: windows::Win32::Foundation::HANDLE) -> Option<Self> {
    let (hid, _) = hid::ContactParser::open(handle)?;
    Some(Self {
      hid,
      last_centroid: None,
      driving: false,
      ignored: false,
      pointer: PointerEpisode::default(),
    })
  }

  fn handle_report(&mut self, report: &[u8]) {
    let Some(frame) = self.hid.frame(report) else {
      return;
    };
    if frame.declared_count >= 2 {
      LAST_NATIVE_CONTACT.with_borrow_mut(|last| *last = Some(Instant::now()));
    }
    let contacts = frame.contacts;
    self.pointer.update(
      contacts.len(),
      contacts.first().map(|contact| (contact.x, contact.y)),
    );
    if contacts.len() != 2 {
      self.last_centroid = None;
      self.ignored = false;
      if self.driving {
        self.driving = false;
        if session::active_input() == Some(InputKind::TrackpadContacts) {
          if let Some(app) = APP.get() {
            session::end(app, false);
          }
        }
      }
      return;
    }
    let centroid = (
      (contacts[0].x + contacts[1].x) * 0.5,
      (contacts[0].y + contacts[1].y) * 0.5,
    );
    let previous = self.last_centroid.replace(centroid);
    let settings = native_settings::snapshot();
    if native_settings::is_down(settings.mouse_modifier) {
      self.ignored = true;
      if self.driving {
        self.driving = false;
        if session::active_input() == Some(InputKind::TrackpadContacts) {
          if let Some(app) = APP.get() {
            session::end(app, true);
          }
        }
      }
    }
    if self.ignored {
      return;
    }
    if !self.driving {
      self.driving = if session::active_input() == Some(InputKind::TrackpadScroll) {
        session::promote_scroll_to_contacts()
      } else {
        begin_session(InputKind::TrackpadContacts)
      };
    }
    let Some((previous_x, previous_y)) = previous else {
      return;
    };
    if !self.driving || session::active_input() != Some(InputKind::TrackpadContacts) {
      return;
    }
    let (delta_x, delta_y) = (centroid.0 - previous_x, centroid.1 - previous_y);
    if (delta_x != 0.0 || delta_y != 0.0) && delta_x.abs() < 250.0 && delta_y.abs() < 250.0 {
      if let Some(app) = APP.get() {
        session::update(
          app,
          delta_x,
          delta_y,
          native_settings::is_down(settings.thirds_modifier),
        );
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use std::time::{Duration, Instant};

  use super::{extend_suppression, mouse_glide_blocked};

  #[test]
  fn only_a_controlled_one_finger_episode_blocks_mouse_glide() {
    assert!(mouse_glide_blocked(true, true));
    assert!(!mouse_glide_blocked(true, false));
    assert!(!mouse_glide_blocked(false, true));
  }

  #[test]
  fn synthesized_wheel_tail_stays_suppressed_until_it_goes_quiet() {
    let now = Instant::now();
    let mut last = Some(now - Duration::from_millis(100));
    assert!(extend_suppression(&mut last, now));
    assert_eq!(last, Some(now));
    assert!(extend_suppression(
      &mut last,
      now + Duration::from_millis(100)
    ));
  }

  #[test]
  fn fallback_reopens_after_a_quiet_gap() {
    let now = Instant::now();
    let mut last = Some(now - Duration::from_millis(200));
    assert!(!extend_suppression(&mut last, now));
  }
}
