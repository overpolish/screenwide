// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tells a touchpad's wheel packets from a mouse wheel's. A touchpad scrolls
//! through the same wheel messages a mouse sends, and only a touchpad's may
//! open a window Glide: a mouse wheel over a title bar keeps scrolling it.

use std::{
  cell::RefCell,
  collections::HashMap,
  sync::atomic::{AtomicBool, Ordering},
};

use windows::{
  core::PCWSTR,
  Win32::{
    Devices::{
      DeviceAndDriverInstallation::{
        CM_Get_DevNode_PropertyW, CM_Get_Device_Interface_PropertyW, CM_Locate_DevNodeW,
        CM_LOCATE_DEVNODE_NORMAL, CR_BUFFER_SMALL, CR_SUCCESS,
      },
      Properties::{
        DEVPKEY_Device_InLocalMachineContainer, DEVPKEY_Device_InstanceId, DEVPROPTYPE,
        DEVPROP_TYPE_BOOLEAN, DEVPROP_TYPE_STRING,
      },
    },
    Foundation::HANDLE,
    UI::Input::{GetRawInputDeviceInfoW, RIDI_DEVICENAME},
  },
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum WheelSource {
  /// Synthesized by the precision touchpad stack, which names no device.
  PrecisionTouchpad,
  /// A pointing device built into the computer, such as an older laptop
  /// touchpad that scrolls through its own driver.
  Integrated,
  /// A device plugged in or paired, or one whose placement could not be read.
  Mouse,
  /// Sent by software, such as a smooth-scrolling utility for a mouse.
  Injected,
}

impl WheelSource {
  pub(super) const fn is_trackpad(self) -> bool {
    matches!(self, Self::PrecisionTouchpad | Self::Integrated)
  }
}

static HOOK_WHEEL_INJECTED: AtomicBool = AtomicBool::new(false);

thread_local! {
  static DEVICES: RefCell<HashMap<isize, WheelSource>> = RefCell::new(HashMap::new());
}

/// Raw input cannot tell injected packets from the touchpad's synthesized
/// ones: both name no device. The low-level hook sees each packet before raw
/// input reports it and can, so it records the answer here.
pub(super) fn note_hook_wheel(injected: bool) {
  HOOK_WHEEL_INJECTED.store(injected, Ordering::Relaxed);
}

pub(super) fn classify(device: HANDLE, hook_active: bool) -> WheelSource {
  if hook_active && HOOK_WHEEL_INJECTED.load(Ordering::Relaxed) {
    return WheelSource::Injected;
  }
  if device.0.is_null() {
    return WheelSource::PrecisionTouchpad;
  }
  DEVICES.with_borrow_mut(|devices| {
    *devices
      .entry(device.0 as isize)
      .or_insert_with(|| placement(device))
  })
}

/// Windows groups a device with the computer it is built into, so a laptop's
/// touchpad sits in the local machine's container and a USB or Bluetooth
/// mouse in its own.
fn placement(device: HANDLE) -> WheelSource {
  let integrated = device_name(device)
    .as_deref()
    .and_then(in_local_machine_container);
  if integrated == Some(true) {
    WheelSource::Integrated
  } else {
    WheelSource::Mouse
  }
}

/// The device's interface path, null-terminated.
fn device_name(device: HANDLE) -> Option<Vec<u16>> {
  let mut length = 0;
  let query = unsafe { GetRawInputDeviceInfoW(Some(device), RIDI_DEVICENAME, None, &mut length) };
  if query == u32::MAX || length == 0 {
    return None;
  }
  let mut name = vec![0_u16; length as usize + 1];
  let read = unsafe {
    GetRawInputDeviceInfoW(
      Some(device),
      RIDI_DEVICENAME,
      Some(name.as_mut_ptr().cast()),
      &mut length,
    )
  };
  if read == u32::MAX {
    return None;
  }
  let end = name.iter().position(|&unit| unit == 0)?;
  name.truncate(end + 1);
  Some(name)
}

fn in_local_machine_container(interface: &[u16]) -> Option<bool> {
  let instance = instance_id(interface)?;
  let mut node = 0;
  let located = unsafe {
    CM_Locate_DevNodeW(
      &mut node,
      PCWSTR(instance.as_ptr()),
      CM_LOCATE_DEVNODE_NORMAL,
    )
  };
  if located != CR_SUCCESS {
    return None;
  }
  let mut kind = DEVPROPTYPE(0);
  let mut value = 0_u8;
  let mut size = 1;
  let read = unsafe {
    CM_Get_DevNode_PropertyW(
      node,
      &DEVPKEY_Device_InLocalMachineContainer,
      &mut kind,
      Some(&mut value),
      &mut size,
      0,
    )
  };
  (read == CR_SUCCESS && kind == DEVPROP_TYPE_BOOLEAN).then_some(value != 0)
}

/// The instance id of the device node behind an interface path.
fn instance_id(interface: &[u16]) -> Option<Vec<u16>> {
  let path = PCWSTR(interface.as_ptr());
  let mut kind = DEVPROPTYPE(0);
  let mut size = 0;
  let query = unsafe {
    CM_Get_Device_Interface_PropertyW(
      path,
      &DEVPKEY_Device_InstanceId,
      &mut kind,
      None,
      &mut size,
      0,
    )
  };
  if query != CR_BUFFER_SMALL || size == 0 {
    return None;
  }
  let mut instance = vec![0_u16; (size as usize).div_ceil(2)];
  let read = unsafe {
    CM_Get_Device_Interface_PropertyW(
      path,
      &DEVPKEY_Device_InstanceId,
      &mut kind,
      Some(instance.as_mut_ptr().cast()),
      &mut size,
      0,
    )
  };
  (read == CR_SUCCESS && kind == DEVPROP_TYPE_STRING).then_some(instance)
}
