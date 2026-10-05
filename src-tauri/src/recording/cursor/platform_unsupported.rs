// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::thread::JoinHandle;

use super::EventSink;

pub(super) fn start(_stop: Arc<AtomicBool>, _sink: EventSink) -> Result<JoinHandle<()>, String> {
  Err("Cursor event recording is not yet implemented on this platform".to_owned())
}

pub(super) fn on_own_window(_x: f64, _y: f64) -> bool {
  false
}
