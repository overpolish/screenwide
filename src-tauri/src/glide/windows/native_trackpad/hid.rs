// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;

use windows::Win32::{
  Devices::HumanInterfaceDevice::{
    HidP_GetButtonCaps, HidP_GetCaps, HidP_GetUsageValue, HidP_GetUsages, HidP_GetValueCaps,
    HidP_Input, HIDP_BUTTON_CAPS, HIDP_CAPS, HIDP_STATUS_SUCCESS, HIDP_VALUE_CAPS,
    PHIDP_PREPARSED_DATA,
  },
  Foundation::HANDLE,
  UI::Input::{GetRawInputDeviceInfoW, RIDI_PREPARSEDDATA},
};

const GENERIC_DESKTOP: u16 = 0x01;
const DIGITIZER: u16 = 0x0d;
const X: u16 = 0x30;
const Y: u16 = 0x31;
const TIP_SWITCH: u16 = 0x42;
const CONTACT_ID: u16 = 0x51;
const CONTACT_COUNT: u16 = 0x54;
const NORMALIZED_SURFACE: f64 = 1_000.0;

#[derive(Clone, Copy)]
struct Axis {
  usage: u16,
  collection: u16,
  logical_min: i32,
  logical_max: i32,
}

#[derive(Clone, Copy)]
struct Contact {
  collection: u16,
  id: Axis,
  x: Axis,
  y: Axis,
}

pub(super) struct ContactPoint {
  pub(super) x: f64,
  pub(super) y: f64,
}

pub(super) struct ContactFrame {
  pub(super) declared_count: usize,
  pub(super) contacts: Vec<ContactPoint>,
}

pub(super) struct ContactParser {
  _storage: Vec<usize>,
  preparsed: PHIDP_PREPARSED_DATA,
  contacts: Vec<Contact>,
  contact_count: Axis,
}

impl ContactParser {
  pub(super) fn open(handle: HANDLE) -> Option<(Self, u16)> {
    let mut byte_count = 0;
    let query =
      unsafe { GetRawInputDeviceInfoW(Some(handle), RIDI_PREPARSEDDATA, None, &mut byte_count) };
    if query == u32::MAX || byte_count == 0 {
      return None;
    }
    let words = (byte_count as usize).div_ceil(std::mem::size_of::<usize>());
    let mut storage = vec![0_usize; words];
    let read = unsafe {
      GetRawInputDeviceInfoW(
        Some(handle),
        RIDI_PREPARSEDDATA,
        Some(storage.as_mut_ptr().cast()),
        &mut byte_count,
      )
    };
    if read == u32::MAX {
      return None;
    }
    let preparsed = PHIDP_PREPARSED_DATA(storage.as_ptr() as isize);
    let mut caps = HIDP_CAPS::default();
    if unsafe { HidP_GetCaps(preparsed, &mut caps) } != HIDP_STATUS_SUCCESS {
      return None;
    }
    let (contacts, contact_count) = collections(preparsed, caps)?;
    Some((
      Self {
        _storage: storage,
        preparsed,
        contacts,
        contact_count,
      },
      caps.InputReportByteLength,
    ))
  }

  pub(super) fn frame(&self, report: &[u8]) -> Option<ContactFrame> {
    let declared_count =
      usize::try_from(usage_value(self.preparsed, self.contact_count, report)?).ok()?;
    let contacts = self
      .contacts
      .iter()
      // In parallel reports only the first Contact Count records are valid;
      // the remaining descriptor slots may retain arbitrary previous bits.
      .take(declared_count.min(self.contacts.len()))
      .filter_map(|contact| {
        if !tip_is_down(self.preparsed, contact.collection, report) {
          return None;
        }
        usage_value(self.preparsed, contact.id, report)?;
        let x = usage_value(self.preparsed, contact.x, report)?;
        let y = usage_value(self.preparsed, contact.y, report)?;
        Some(ContactPoint {
          x: normalize(x, contact.x),
          y: normalize(y, contact.y),
        })
      })
      .collect();
    Some(ContactFrame {
      declared_count,
      contacts,
    })
  }
}

fn collections(preparsed: PHIDP_PREPARSED_DATA, caps: HIDP_CAPS) -> Option<(Vec<Contact>, Axis)> {
  let mut values = vec![HIDP_VALUE_CAPS::default(); caps.NumberInputValueCaps as usize];
  let mut value_count = caps.NumberInputValueCaps;
  if unsafe { HidP_GetValueCaps(HidP_Input, values.as_mut_ptr(), &mut value_count, preparsed) }
    != HIDP_STATUS_SUCCESS
  {
    return None;
  }
  values.truncate(value_count as usize);
  let mut buttons = vec![HIDP_BUTTON_CAPS::default(); caps.NumberInputButtonCaps as usize];
  let mut button_count = caps.NumberInputButtonCaps;
  if button_count > 0
    && unsafe {
      HidP_GetButtonCaps(
        HidP_Input,
        buttons.as_mut_ptr(),
        &mut button_count,
        preparsed,
      )
    } != HIDP_STATUS_SUCCESS
  {
    return None;
  }
  buttons.truncate(button_count as usize);

  let mut axes: HashMap<u16, (Option<Axis>, Option<Axis>)> = HashMap::new();
  let mut ids = HashMap::new();
  let mut contact_count = None;
  for cap in &values {
    if cap.UsagePage == DIGITIZER && value_contains(cap, CONTACT_COUNT) {
      contact_count = Some(axis(cap, CONTACT_COUNT));
    }
    if cap.UsagePage == DIGITIZER && value_contains(cap, CONTACT_ID) {
      ids.insert(cap.LinkCollection, axis(cap, CONTACT_ID));
    }
    if cap.UsagePage == GENERIC_DESKTOP {
      for usage in [X, Y] {
        if !value_contains(cap, usage) {
          continue;
        }
        let field = axis(cap, usage);
        let pair = axes.entry(cap.LinkCollection).or_default();
        if usage == X {
          pair.0 = Some(field);
        } else {
          pair.1 = Some(field);
        }
      }
    }
  }
  // Button-cap order defines which records Contact Count makes valid.
  let tips = buttons
    .iter()
    .filter(|cap| cap.UsagePage == DIGITIZER && button_contains(cap, TIP_SWITCH))
    .map(|cap| cap.LinkCollection)
    .fold(Vec::new(), |mut ordered, collection| {
      if !ordered.contains(&collection) {
        ordered.push(collection);
      }
      ordered
    });
  let contacts = tips
    .into_iter()
    .filter_map(|collection| {
      let (x, y) = axes.get(&collection)?;
      Some(Contact {
        collection,
        id: *ids.get(&collection)?,
        x: (*x)?,
        y: (*y)?,
      })
    })
    .collect::<Vec<_>>();
  (!contacts.is_empty()).then_some((contacts, contact_count?))
}

fn axis(cap: &HIDP_VALUE_CAPS, usage: u16) -> Axis {
  Axis {
    usage,
    collection: cap.LinkCollection,
    logical_min: cap.LogicalMin,
    logical_max: cap.LogicalMax,
  }
}

fn value_contains(cap: &HIDP_VALUE_CAPS, usage: u16) -> bool {
  unsafe {
    if cap.IsRange {
      let r = cap.Anonymous.Range;
      usage >= r.UsageMin && usage <= r.UsageMax
    } else {
      cap.Anonymous.NotRange.Usage == usage
    }
  }
}

fn button_contains(cap: &HIDP_BUTTON_CAPS, usage: u16) -> bool {
  unsafe {
    if cap.IsRange {
      let r = cap.Anonymous.Range;
      usage >= r.UsageMin && usage <= r.UsageMax
    } else {
      cap.Anonymous.NotRange.Usage == usage
    }
  }
}

fn tip_is_down(preparsed: PHIDP_PREPARSED_DATA, collection: u16, report: &[u8]) -> bool {
  let mut usages = [0_u16; 16];
  let mut count = usages.len() as u32;
  let mut report = report.to_vec();
  let status = unsafe {
    HidP_GetUsages(
      HidP_Input,
      DIGITIZER,
      Some(collection),
      usages.as_mut_ptr(),
      &mut count,
      preparsed,
      &mut report,
    )
  };
  status == HIDP_STATUS_SUCCESS && usages[..count as usize].contains(&TIP_SWITCH)
}

fn usage_value(preparsed: PHIDP_PREPARSED_DATA, axis: Axis, report: &[u8]) -> Option<u32> {
  let mut value = 0;
  (unsafe {
    HidP_GetUsageValue(
      HidP_Input,
      if matches!(axis.usage, X | Y) {
        GENERIC_DESKTOP
      } else {
        DIGITIZER
      },
      Some(axis.collection),
      axis.usage,
      &mut value,
      preparsed,
      report,
    )
  } == HIDP_STATUS_SUCCESS)
    .then_some(value)
}

fn normalize(value: u32, axis: Axis) -> f64 {
  let span = f64::from(axis.logical_max - axis.logical_min).max(1.0);
  (f64::from(value) - f64::from(axis.logical_min)) * NORMALIZED_SURFACE / span
}

#[cfg(test)]
mod tests {
  use super::{normalize, Axis};

  #[test]
  fn normalizes_device_coordinates_to_a_stable_surface() {
    let axis = Axis {
      usage: 0,
      collection: 0,
      logical_min: 100,
      logical_max: 2100,
    };
    assert_eq!(normalize(100, axis), 0.0);
    assert_eq!(normalize(1100, axis), 500.0);
    assert_eq!(normalize(2100, axis), 1000.0);
  }
}
