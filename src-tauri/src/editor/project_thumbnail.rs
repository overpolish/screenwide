// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The project's `preview.png`: the picture the browser shows for it.
//!
//! Kept in the project, as Motion and Final Cut keep theirs, so a project
//! copied to another computer shows its edit there before it is opened.
//!
//! A recording the editor has had open shows its edit, composed by the editor
//! as it changes. One it has not, such as a saved replay, shows a frame of
//! the movie; an audio recording shows its ribbon.

pub(crate) mod edited;
#[cfg(any(target_os = "macos", target_os = "windows"))]
mod waveform;

use super::*;

/// Twice the widest card, so it stays sharp on a Retina display.
const PREVIEW_WIDTH: u32 = 640;

/// The preview of the project whose manifest is `file`, made on first ask
/// for a project that has none. None when there is nothing to show.
pub fn project_thumbnail(file: &Path) -> Result<Option<PathBuf>, String> {
  let preview = crate::project::preview_path(file);
  if preview.is_file() {
    return Ok(Some(preview));
  }
  let manifest = crate::project::read(file)?;
  if manifest.kind == crate::project::ProjectKind::Screenshot {
    return bottom_layer(file, &manifest, &preview);
  }
  let recording = manifest.recorded()?;
  let movie = crate::project::resolve(file, &recording.media.primary)?;
  // The middle stands for the recording better than its start, which is
  // often the recording bar going away. A project from before its length was
  // kept is measured.
  let duration_ms = recording
    .duration_ms
    .or_else(|| media_preview::duration_ms(&movie));
  if recording.primary_kind == PrimaryRecordingKind::Audio {
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    return waveform::write(&movie, duration_ms, &preview).map(|()| Some(preview));
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    return Ok(None);
  }
  movie_frame(&movie, duration_ms, &preview)?;
  Ok(Some(preview))
}

/// A screenshot the editor has not drawn a preview for yet: its bottom
/// picture, as it was taken.
fn bottom_layer(
  file: &Path,
  manifest: &crate::project::Manifest,
  preview: &Path,
) -> Result<Option<PathBuf>, String> {
  let Some(layer) = manifest
    .screenshot
    .as_ref()
    .and_then(|screenshot| screenshot.layers.first())
  else {
    return Ok(None);
  };
  let picture =
    image::open(crate::project::resolve(file, &layer.image)?).map_err(|error| error.to_string())?;
  let smaller = picture.thumbnail(PREVIEW_WIDTH, u32::MAX);
  write_atomically(preview, false, |partial| {
    smaller
      .save_with_format(partial, image::ImageFormat::Png)
      .map_err(|error| error.to_string())
  })?;
  Ok(Some(preview.to_path_buf()))
}

/// Writes `target` through a neighbouring file, so a reader never sees half
/// a picture. With `replace` false it is written only if absent: a frame made
/// while the editor composed the edit must not land over it.
fn write_atomically(
  target: &Path,
  replace: bool,
  write: impl FnOnce(&Path) -> Result<(), String>,
) -> Result<(), String> {
  let partial = target.with_extension("part.png");
  if let Err(error) = write(&partial) {
    let _ = std::fs::remove_file(&partial);
    return Err(error);
  }
  let placed = if replace {
    std::fs::rename(&partial, target)
  } else {
    // A hard link cannot replace a file, so it places the picture only where
    // there is none, in one step.
    let linked = std::fs::hard_link(&partial, target);
    let _ = std::fs::remove_file(&partial);
    match linked {
      Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => Ok(()),
      other => other,
    }
  };
  placed.map_err(|error| error.to_string())
}

/// The movie's frame at its middle.
fn movie_frame(movie: &Path, duration_ms: Option<u64>, target: &Path) -> Result<(), String> {
  let at_seconds = duration_ms.map_or(1.0, |ms| ms as f64 / 2_000.0);
  write_atomically(target, false, |partial| {
    let output = media_preview::ffmpeg_command()
      .args([
        "-hide_banner",
        "-loglevel",
        "error",
        "-nostdin",
        "-y",
        "-ss",
      ])
      .arg(format!("{at_seconds:.3}"))
      .arg("-i")
      .arg(movie)
      .args(["-frames:v", "1", "-vf"])
      .arg(format!("scale={PREVIEW_WIDTH}:-2"))
      .arg(partial)
      .output()
      .map_err(|error| format!("FFmpeg could not be started: {error}"))?;
    if output.status.success() && partial.is_file() {
      Ok(())
    } else {
      Err(String::from_utf8_lossy(&output.stderr).trim().to_owned())
    }
  })
}

#[cfg(test)]
mod tests {
  use super::*;

  fn scratch(name: &str) -> PathBuf {
    let directory = std::env::temp_dir()
      .join("screenwide-tests")
      .join(format!("preview-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).unwrap();
    directory
  }

  fn write_bytes(bytes: &'static [u8]) -> impl FnOnce(&Path) -> Result<(), String> {
    move |path| std::fs::write(path, bytes).map_err(|error| error.to_string())
  }

  #[test]
  fn a_frame_made_after_the_edit_does_not_replace_it() {
    let preview = scratch("kept").join("preview.png");
    write_atomically(&preview, true, write_bytes(b"edit")).unwrap();
    write_atomically(&preview, false, write_bytes(b"frame")).unwrap();
    assert_eq!(std::fs::read(&preview).unwrap(), b"edit");
    assert!(!preview.with_extension("part.png").exists());
  }

  #[test]
  fn the_edit_replaces_a_frame() {
    let preview = scratch("replaced").join("preview.png");
    write_atomically(&preview, false, write_bytes(b"frame")).unwrap();
    write_atomically(&preview, true, write_bytes(b"edit")).unwrap();
    assert_eq!(std::fs::read(&preview).unwrap(), b"edit");
  }
}
