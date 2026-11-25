// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

mod export_command;
mod export_credits;
mod finished;
mod format_pass;
mod screenshot_formats;
pub use export_command::save_export;
pub use finished::{copy_export_file, drag_export_file, put_away_export, reveal_export_file};
pub use format_pass::save_format_exports;
pub use screenshot_formats::save_screenshot_formats;

use super::*;

mod export_job;
mod lifecycle;
mod recording_file;
mod write_failure;

pub(crate) use finished::{drag_out, DragLook};
pub(crate) use lifecycle::is_exporting;
pub(crate) use write_failure::write_failure;

pub use export_command::{__cmd__save_export, __tauri_command_name_save_export};
pub use finished::{
  __cmd__copy_export_file, __cmd__drag_export_file, __cmd__put_away_export,
  __cmd__reveal_export_file, __tauri_command_name_copy_export_file,
  __tauri_command_name_drag_export_file, __tauri_command_name_put_away_export,
  __tauri_command_name_reveal_export_file,
};
pub use format_pass::{__cmd__save_format_exports, __tauri_command_name_save_format_exports};
pub(super) use recording_file::{
  delivered_extension, save_primary_recording, save_recording_copy, save_selected_recording_copy,
  scale_percent, PrimaryRecordingSaveRequest,
};
pub use screenshot_formats::{
  __cmd__save_screenshot_formats, __tauri_command_name_save_screenshot_formats,
};
