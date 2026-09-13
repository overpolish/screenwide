// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#[path = "save/export_command.rs"]
mod export_command;
pub use export_command::save_export;

use super::*;

mod cursor;
mod export_job;
mod lifecycle;
mod location;
mod recording_file;

pub(super) use recording_file::{
  delivered_extension, save_primary_recording, save_recording_copy, save_selected_recording_copy,
  scale_percent, PrimaryRecordingSaveRequest,
};

#[cfg(test)]
pub(super) use recording_file::{save_recording, save_selected_recording};

pub use export_command::{__cmd__save_export, __tauri_command_name_save_export};
