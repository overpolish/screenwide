// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A stretch of an exported video as a looping GIF. The look is fixed, so
//! nobody has to tune one: 20 frames a second, which GIF's hundredths of a
//! second time exactly; the longer side 640 px at most and never larger than
//! the video; no sound. A GIF frame holds 256 colours, so a palette is made
//! from the stretch first, then each frame is mapped onto it, only where it
//! changed, which keeps screen recordings small.

use super::*;
use screenwide_sidecar::ffmpeg_command;

const GIF_FRAMES_PER_SECOND: u32 = 20;
const GIF_LONGER_SIDE: u32 = 640;
/// The longest stretch a GIF is made of. GIFs are large, so the stretch is
/// one drawn on the timeline rather than the whole recording.
pub const GIF_LONGEST_MS: u64 = 15_000;

/// The frames a GIF is made of: its rate, then its size, the longer side
/// at most [`GIF_LONGER_SIDE`] and never enlarged, the other kept in
/// proportion.
fn frames_filter() -> String {
  let side = GIF_LONGER_SIDE;
  format!(
    "fps={GIF_FRAMES_PER_SECOND},\
     scale='if(gte(iw,ih),min({side},iw),-1)':'if(gte(iw,ih),-1,min({side},ih))':flags=lanczos"
  )
}

fn seconds(ms: u64) -> String {
  format!("{:.3}", ms as f64 / 1_000.0)
}

/// The stretch of `source` from `start_ms`, `duration_ms` long.
fn stretch_input(source: &Path, start_ms: u64, duration_ms: u64) -> Vec<OsString> {
  let mut args: Vec<OsString> = ["-ss".into(), seconds(start_ms).into()].into();
  args.extend(["-t".into(), seconds(duration_ms).into(), "-i".into()]);
  args.push(source.as_os_str().to_owned());
  args
}

fn quiet() -> std::process::Command {
  let mut command = ffmpeg_command();
  command.args(["-hide_banner", "-nostdin", "-loglevel", "error", "-y"]);
  command
}

fn gif_error(stderr: &[u8]) -> String {
  const MESSAGE: &str = "FFmpeg could not make the GIF";
  let detail = String::from_utf8_lossy(stderr);
  let detail = detail.trim();
  if detail.is_empty() {
    return MESSAGE.to_owned();
  }
  let tail = detail
    .char_indices()
    .rev()
    .nth(OUTPUT_ERROR_DETAIL - 1)
    .map_or(detail, |(index, _)| &detail[index..]);
  format!("{MESSAGE}: {tail}")
}

/// Makes `destination` from `duration_ms` of `source` starting at
/// `start_ms`, reporting how far into the stretch it has got. Nothing is
/// left behind if it is cancelled or fails.
pub fn encode_gif(
  source: &Path,
  destination: &Path,
  [start_ms, duration_ms]: [u64; 2],
  cancelled: &AtomicBool,
  on_progress: &mut dyn FnMut(u64),
) -> Result<ExportRunResult, String> {
  if duration_ms == 0 || duration_ms > GIF_LONGEST_MS {
    return Err(format!(
      "A GIF is made of up to {} seconds",
      GIF_LONGEST_MS / 1_000
    ));
  }
  let temporary = remux_temp_path(destination);
  // Raw video in NUT: the bundled macOS FFmpeg has no PNG encoder.
  let palette = temporary.with_extension("palette.nut");
  let result = encode_through(
    source,
    [&palette, &temporary],
    [start_ms, duration_ms],
    cancelled,
    on_progress,
  );
  let _ = std::fs::remove_file(&palette);
  let result = result.and_then(|run| match run {
    ExportRunResult::Completed => std::fs::rename(&temporary, destination)
      .map(|()| ExportRunResult::Completed)
      .map_err(|error| format!("The GIF could not be put in place: {error}")),
    ExportRunResult::Cancelled => Ok(ExportRunResult::Cancelled),
  });
  if !matches!(result, Ok(ExportRunResult::Completed)) {
    let _ = std::fs::remove_file(&temporary);
  }
  result
}

fn encode_through(
  source: &Path,
  [palette, temporary]: [&Path; 2],
  [start_ms, duration_ms]: [u64; 2],
  cancelled: &AtomicBool,
  on_progress: &mut dyn FnMut(u64),
) -> Result<ExportRunResult, String> {
  let made = quiet()
    .args(stretch_input(source, start_ms, duration_ms))
    .args(["-an", "-vf"])
    .arg(format!("{},palettegen=stats_mode=diff", frames_filter()))
    .args(["-frames:v", "1", "-c:v", "rawvideo", "-f", "nut"])
    .arg(palette)
    .stdout(Stdio::null())
    .output()
    .map_err(|error| format!("FFmpeg could not be started: {error}"))?;
  if !made.status.success() || !holds_bytes(palette) {
    return Err(gif_error(&made.stderr));
  }
  if cancelled.load(Ordering::Acquire) {
    return Ok(ExportRunResult::Cancelled);
  }

  let mut child = quiet()
    .args(stretch_input(source, start_ms, duration_ms))
    .arg("-i")
    .arg(palette)
    .arg("-lavfi")
    .arg(format!(
      "[0:v]{}[frames];[frames][1:v]paletteuse=dither=sierra2_4a:diff_mode=rectangle",
      frames_filter()
    ))
    .args([
      "-an",
      "-loop",
      "0",
      "-f",
      "gif",
      "-progress",
      "pipe:1",
      "-nostats",
    ])
    .arg(temporary)
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()
    .map_err(|error| format!("FFmpeg could not be started: {error}"))?;
  let stderr_reader = process::read_stderr(&mut child)?;
  let stdout = child
    .stdout
    .take()
    .ok_or_else(|| "FFmpeg did not expose its progress output".to_owned())?;
  for line in BufReader::new(stdout).lines().map_while(Result::ok) {
    if cancelled.load(Ordering::Acquire) {
      let _ = child.kill();
      break;
    }
    if let Some(milliseconds) = process::progress_milliseconds(&line) {
      on_progress(milliseconds.min(duration_ms));
    }
  }
  let status = child
    .wait()
    .map_err(|error| format!("FFmpeg could not be completed: {error}"))?;
  let stderr = stderr_reader.join().unwrap_or_default();
  if cancelled.load(Ordering::Acquire) {
    return Ok(ExportRunResult::Cancelled);
  }
  if !status.success() || !holds_bytes(temporary) {
    return Err(gif_error(&stderr));
  }
  Ok(ExportRunResult::Completed)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn stretches_longer_than_the_cap_are_refused_before_ffmpeg_runs() {
    let cancelled = AtomicBool::new(false);
    let result = encode_gif(
      Path::new("missing.mp4"),
      Path::new("missing.gif"),
      [0, GIF_LONGEST_MS + 1],
      &cancelled,
      &mut |_| {},
    );
    assert!(result.is_err());
  }
}
