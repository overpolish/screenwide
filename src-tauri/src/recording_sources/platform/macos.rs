// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#[cfg(target_os = "macos")]
use std::path::{Path, PathBuf};

#[cfg(target_os = "macos")]
use {
  cidre::{cg, sc},
  objc2::AnyThread,
  objc2_app_kit::{
    NSApplicationActivationPolicy, NSBitmapImageFileType, NSBitmapImageRep, NSRunningApplication,
  },
  objc2_foundation::{NSDictionary, NSString},
  std::collections::HashSet,
};

#[cfg(target_os = "macos")]
pub struct AudioApplication {
  pub id: String,
  pub label: String,
  pub pid: u32,
}

#[cfg(target_os = "macos")]
pub async fn audio_applications() -> Result<Vec<AudioApplication>, String> {
  let current_pid = std::process::id();
  let content = sc::ShareableContent::current()
    .await
    .map_err(|error| error.to_string())?;

  Ok(
    content
      .apps()
      .iter()
      .filter_map(|application| {
        let pid = u32::try_from(application.process_id()).ok()?;
        let running_application =
          NSRunningApplication::runningApplicationWithProcessIdentifier(pid as i32)?;
        if running_application.activationPolicy() != NSApplicationActivationPolicy::Regular {
          return None;
        }
        let id = application.bundle_id().to_string();
        let label = application.app_name().to_string();
        if pid == current_pid || id.trim().is_empty() || label.trim().is_empty() {
          return None;
        }
        Some(AudioApplication {
          id,
          label: label.trim().to_owned(),
          pid,
        })
      })
      .collect(),
  )
}

#[cfg(target_os = "macos")]
pub fn app_icon(cache_dir: &Path, pid: u32) -> Option<PathBuf> {
  unsafe {
    let running_app = NSRunningApplication::runningApplicationWithProcessIdentifier(pid as i32)?;
    let bundle_id = running_app.bundleIdentifier()?.to_string();
    let path = cache_dir.join(format!("app-{}.png", sanitize_filename(&bundle_id)));
    if path.exists() {
      return Some(path);
    }

    let icon = running_app.icon()?;
    let cg_image = icon.CGImageForProposedRect_context_hints(std::ptr::null_mut(), None, None)?;
    let bitmap = NSBitmapImageRep::initWithCGImage(NSBitmapImageRep::alloc(), &cg_image);
    let properties = NSDictionary::new();
    let png = bitmap.representationUsingType_properties(NSBitmapImageFileType::PNG, &properties)?;
    let path_string = NSString::from_str(&path.to_string_lossy());
    png
      .writeToFile_atomically(&path_string, true)
      .then_some(path)
  }
}

#[cfg(target_os = "macos")]
pub fn selectable_window_ids() -> Option<HashSet<u32>> {
  let options = cg::WindowListOpt::ON_SCREEN_ONLY | cg::WindowListOpt::EXCLUDE_DESKTOP_ELEMENTS;
  let windows = cg::WindowList::info(options, cg::WINDOW_ID_NULL)?;
  Some(
    windows
      .iter()
      .filter_map(|window| {
        let layer = window
          .get(cg::window_keys::layer())?
          .try_as_number()?
          .to_i32()?;
        if layer != 0 {
          return None;
        }
        window
          .get(cg::window_keys::number())?
          .try_as_number()?
          .to_i32()
          .map(|id| id as u32)
      })
      .collect(),
  )
}

#[cfg(target_os = "macos")]
fn sanitize_filename(value: &str) -> String {
  value
    .chars()
    .map(|character| {
      if character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.') {
        character
      } else {
        '_'
      }
    })
    .collect()
}
