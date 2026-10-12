// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The programs bundled beside the app: FFmpeg, FFprobe and the speech-to-text
//! helper. Finding them, and starting them without a console window.

use std::path::{Path, PathBuf};
use std::process::Command;

/// Where a package manager puts FFmpeg and its siblings. A bundled app
/// inherits its environment from launchd rather than from a shell, so `PATH`
/// alone finds nothing that was installed by Homebrew or MacPorts.
#[cfg(not(windows))]
const TOOL_SEARCH_DIRECTORIES: &[&str] = &[
  "/opt/homebrew/bin",
  "/usr/local/bin",
  "/opt/local/bin",
  "/usr/bin",
];

#[cfg(windows)]
const TOOL_SEARCH_DIRECTORIES: &[&str] = &[];

fn tool_path(name: &str) -> PathBuf {
  let executable = if cfg!(windows) {
    format!("{name}.exe")
  } else {
    name.to_owned()
  };
  // Alongside the app first: a bundled copy is the one this build was tested
  // against.
  if let Ok(current) = std::env::current_exe() {
    if let Some(directory) = current.parent() {
      let bundled = directory.join(&executable);
      if bundled.is_file() {
        return bundled;
      }
    }
  }
  for directory in TOOL_SEARCH_DIRECTORIES {
    let path = Path::new(directory).join(&executable);
    if path.is_file() {
      return path;
    }
  }

  PathBuf::from(executable)
}

pub fn ffmpeg_path() -> PathBuf {
  tool_path("ffmpeg")
}

/// FFprobe is only ever used to *check* a file just written, never to produce
/// one. Everything still works without it; the app's export check says what
/// is given up when it is missing.
pub fn ffprobe_path() -> PathBuf {
  tool_path("ffprobe")
}

/// FFmpeg, to be started without a console window. A Windows release build is
/// a GUI-subsystem process, so each console program it starts is given a
/// console of its own: that flashes a window, and on the launch path it held
/// the main thread for over five seconds where the run itself takes 40 ms.
pub fn ffmpeg_command() -> Command {
  without_console(ffmpeg_path())
}

/// FFprobe, started as [`ffmpeg_command`] starts FFmpeg.
pub fn ffprobe_command() -> Command {
  without_console(ffprobe_path())
}

/// Screenwide's speech-to-text helper, bundled beside the app as FFmpeg is
/// and started as it is.
pub fn transcriber_command() -> Command {
  without_console(tool_path("screenwide-transcriber"))
}

fn without_console(program: PathBuf) -> Command {
  #[cfg_attr(not(windows), allow(unused_mut))]
  let mut command = Command::new(program);
  #[cfg(windows)]
  {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    command.creation_flags(CREATE_NO_WINDOW);
  }
  command
}
