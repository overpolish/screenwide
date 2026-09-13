// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

pub(super) fn mux_file(
  destination: &Path,
  duration_ms: u64,
  files: AudioFiles,
  include_video: bool,
) -> Result<(), String> {
  let parent = destination
    .parent()
    .ok_or_else(|| "The recording has no directory".to_owned())?;
  let stem = destination
    .file_stem()
    .and_then(|value| value.to_str())
    .unwrap_or("recording");
  let output = parent.join(format!("{stem}.audio-mux.mp4"));
  let mut command = Command::new(crate::editor::ffmpeg_path());
  command.args(["-hide_banner", "-loglevel", "error", "-nostdin", "-y"]);
  if include_video {
    command.arg("-i").arg(destination);
  }
  let mut input_index = usize::from(include_video);
  let mut system_inputs = Vec::new();
  for source in files
    .system
    .iter()
    .filter(|source| source.path.metadata().is_ok_and(|m| m.len() > 0))
  {
    add_raw_input(&mut command, source);
    system_inputs.push((input_index, source.first_offset_ms));
    input_index += 1;
  }
  let microphone_input = files.microphone.as_ref().and_then(|source| {
    source
      .path
      .metadata()
      .ok()
      .filter(|metadata| metadata.len() > 0)?;
    add_raw_input(&mut command, source);
    let value = (input_index, source.first_offset_ms);
    input_index += 1;
    Some(value)
  });
  let duration = format!("{}.{:03}", duration_ms / 1_000, duration_ms % 1_000);
  let mut filter = String::new();
  let mut maps = Vec::new();
  if files.has_system_audio {
    append_track_filter(
      &mut command,
      &mut input_index,
      &mut filter,
      &system_inputs,
      "system",
      &duration,
    );
    maps.push(("[system]", "System audio"));
  }
  if files.has_microphone {
    let inputs = microphone_input.into_iter().collect::<Vec<_>>();
    append_track_filter(
      &mut command,
      &mut input_index,
      &mut filter,
      &inputs,
      "microphone",
      &duration,
    );
    maps.push(("[microphone]", "Microphone"));
  }
  if include_video {
    command.args(["-map", "0:v:0", "-c:v", "copy"]);
  }
  if !filter.is_empty() {
    command.args(["-filter_complex", filter.trim_end_matches(';')]);
  }
  for (position, (map, title)) in maps.iter().enumerate() {
    command.args(["-map", map]);
    command.arg(format!("-metadata:s:a:{position}"));
    command.arg(format!("title={title}"));
  }
  command.args([
    "-c:a",
    "aac",
    "-b:a",
    "192k",
    "-t",
    &duration,
    "-movflags",
    "+faststart",
  ]);
  command.arg(&output);
  let result = command
    .output()
    .map_err(|error| format!("FFmpeg could not mux recording audio: {error}"))?;
  if !result.status.success() {
    return Err(format!(
      "FFmpeg could not finish recording audio: {}",
      String::from_utf8_lossy(&result.stderr).trim()
    ));
  }
  if include_video {
    let backup = parent.join(format!("{stem}.video-backup.mp4"));
    fs::rename(destination, &backup).map_err(|error| error.to_string())?;
    if let Err(error) = fs::rename(&output, destination) {
      let _ = fs::rename(&backup, destination);
      return Err(error.to_string());
    }
    let _ = fs::remove_file(backup);
  } else {
    fs::rename(&output, destination).map_err(|error| error.to_string())?;
  }
  for source in files.system.into_iter().chain(files.microphone) {
    let _ = fs::remove_file(source.path);
  }
  Ok(())
}

pub(super) fn add_raw_input(command: &mut Command, source: &RawSource) {
  command.args(["-f", "f32le", "-ar"]);
  command.arg(source.sample_rate.to_string());
  command.arg("-ac");
  command.arg(source.channels.to_string());
  command.arg("-i");
  command.arg(&source.path);
}

pub(super) fn append_track_filter(
  command: &mut Command,
  input_index: &mut usize,
  filter: &mut String,
  inputs: &[(usize, u64)],
  label: &str,
  duration: &str,
) {
  if inputs.is_empty() {
    command.args(["-f", "lavfi", "-i", "anullsrc=r=48000:cl=stereo"]);
    let index = *input_index;
    *input_index += 1;
    filter.push_str(&format!("[{index}:a]atrim=duration={duration}[{label}];"));
    return;
  }
  for (position, (index, delay)) in inputs.iter().enumerate() {
    filter.push_str(&format!(
      "[{index}:a]aresample=48000,aformat=sample_fmts=fltp:channel_layouts=stereo,adelay={delay}:all=1,apad,atrim=duration={duration}[{label}{position}];"
    ));
  }
  if inputs.len() == 1 {
    filter.push_str(&format!("[{label}0]anull[{label}];"));
  } else {
    for position in 0..inputs.len() {
      filter.push_str(&format!("[{label}{position}]"));
    }
    filter.push_str(&format!(
      "amix=inputs={}:duration=longest:normalize=0,alimiter=limit=0.98[{label}];",
      inputs.len()
    ));
  }
}
