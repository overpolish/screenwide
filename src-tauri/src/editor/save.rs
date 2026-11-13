// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

mod export_command;
mod export_credits;
mod finished;
pub use export_command::save_export;
pub use finished::{copy_export_file, drag_export_file, put_away_export, reveal_export_file};

use super::*;

mod export_job;
mod lifecycle;
mod recording_file;

pub(crate) use finished::{drag_out, DragLook};
pub(crate) use lifecycle::is_exporting;

pub use export_command::{__cmd__save_export, __tauri_command_name_save_export};
pub use finished::{
  __cmd__copy_export_file, __cmd__drag_export_file, __cmd__put_away_export,
  __cmd__reveal_export_file, __tauri_command_name_copy_export_file,
  __tauri_command_name_drag_export_file, __tauri_command_name_put_away_export,
  __tauri_command_name_reveal_export_file,
};
pub(super) use recording_file::{
  delivered_extension, save_primary_recording, save_recording_copy, save_selected_recording_copy,
  scale_percent, PrimaryRecordingSaveRequest,
};
