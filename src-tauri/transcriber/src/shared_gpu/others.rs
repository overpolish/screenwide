// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Whether another program is using the GPU heavily: the busiest program's
//! share of an engine shader work runs on, where models run too, from Windows'
//! GPU engine counters, the ones Task Manager's GPU column reads. One program
//! at a time, since a game takes a large share by itself, while the everyday
//! programs on a desktop each take a little and together can take a quarter
//! of an engine. Screenwide does not count, nor does the desktop window
//! manager, which draws the desktop whatever else runs and is not taken down
//! by a reset. Elsewhere than Windows the GPU is never taken to be busy.

// The counters are read on Windows only; the arithmetic over them is kept
// apart from the reading so it is tested everywhere.
#[cfg(any(windows, test))]
use std::collections::HashMap;

/// The busiest other program's share of an engine, in percent, from which a
/// model steps aside, and below which it comes back. Apart, so a load
/// hovering near one does not move the work back and forth.
#[cfg(any(windows, test))]
const BUSY_FROM: f64 = 30.0;
#[cfg(any(windows, test))]
const QUIET_BELOW: f64 = 15.0;

pub(super) fn busy() -> bool {
  #[cfg(windows)]
  return counters::busy();
  #[cfg(not(windows))]
  false
}

/// Whether the GPU counts as busy now, given whether it did and the busiest
/// other program's share.
#[cfg(any(windows, test))]
fn settle(was_busy: bool, load: f64) -> bool {
  load >= if was_busy { QUIET_BELOW } else { BUSY_FROM }
}

/// One engine of one program, as Windows names its counter:
/// `pid_1234_luid_0x00000000_0x0000C2A1_phys_0_eng_0_engtype_3D`.
#[cfg(any(windows, test))]
struct Instance<'a> {
  pid: u32,
  /// The adapter and engine, the same for every program on it.
  engine: &'a str,
  kind: &'a str,
}

#[cfg(any(windows, test))]
fn parse(name: &str) -> Option<Instance<'_>> {
  let (pid, rest) = name.strip_prefix("pid_")?.split_once('_')?;
  let (engine, kind) = rest.split_once("_engtype_")?;
  Some(Instance {
    pid: pid.parse().ok()?,
    engine,
    kind,
  })
}

/// The busiest program's share of one 3D or compute engine, in percent, from
/// each engine instance's reading, leaving out the `ignored` programs.
#[cfg(any(windows, test))]
fn others_load<'a>(readings: impl IntoIterator<Item = (&'a str, f64)>, ignored: &[u32]) -> f64 {
  let mut shares: HashMap<(u32, &str), f64> = HashMap::new();
  for (name, value) in readings {
    let Some(instance) = parse(name) else {
      continue;
    };
    let kind = instance.kind.to_ascii_lowercase();
    // Shader work runs on 3D engines, which NVIDIA names Graphics, and on
    // compute engines; copy, video and security engines are left out.
    let models_run_there = ["3d", "graphics", "compute", "cuda"]
      .iter()
      .any(|shaders| kind.starts_with(shaders));
    if models_run_there && !ignored.contains(&instance.pid) {
      *shares.entry((instance.pid, instance.engine)).or_default() += value;
    }
  }
  shares.into_values().fold(0.0, f64::max)
}

#[cfg(windows)]
mod counters {
  use std::sync::{Mutex, PoisonError};
  use std::time::{Duration, Instant};

  use windows::core::w;
  use windows::Win32::Foundation::CloseHandle;
  use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
  };
  use windows::Win32::System::Performance::{
    PdhAddEnglishCounterW, PdhCollectQueryData, PdhGetFormattedCounterArrayW, PdhOpenQueryW,
    PDH_CSTATUS_NEW_DATA, PDH_CSTATUS_VALID_DATA, PDH_FMT_COUNTERVALUE_ITEM_W, PDH_FMT_DOUBLE,
    PDH_HCOUNTER, PDH_HQUERY, PDH_MORE_DATA,
  };
  use windows::Win32::System::Threading::GetCurrentProcessId;

  use super::{others_load, settle};

  /// How long a reading stands: a model stepping frame by frame asks far
  /// more often than the counters are worth reading.
  const READING_LASTS: Duration = Duration::from_millis(500);
  /// The counters measure use between two readings, so the first is followed
  /// by a second this much later.
  const FIRST_SPAN: Duration = Duration::from_millis(200);

  struct Counters {
    query: PDH_HQUERY,
    counter: PDH_HCOUNTER,
    /// The helper, the app that started it and the desktop window manager,
    /// whose use does not count.
    ignored: Vec<u32>,
    read_at: Instant,
    busy: bool,
  }

  // Safety: the handles are only used under the lock below.
  unsafe impl Send for Counters {}

  enum State {
    Unopened,
    /// The counters cannot be read here, and the GPU is never taken to be busy.
    Unavailable,
    Open(Counters),
  }

  static STATE: Mutex<State> = Mutex::new(State::Unopened);

  pub(super) fn busy() -> bool {
    let mut state = STATE.lock().unwrap_or_else(PoisonError::into_inner);
    if matches!(*state, State::Unopened) {
      *state = Counters::open().map_or(State::Unavailable, State::Open);
    }
    let State::Open(counters) = &mut *state else {
      return false;
    };
    if counters.read_at.elapsed() >= READING_LASTS {
      if let Some(load) = counters.read() {
        counters.busy = settle(counters.busy, load);
      }
      counters.read_at = Instant::now();
    }
    counters.busy
  }

  impl Counters {
    fn open() -> Option<Self> {
      let mut query = PDH_HQUERY::default();
      let mut counter = PDH_HCOUNTER::default();
      unsafe {
        if PdhOpenQueryW(None, 0, &mut query) != 0
          || PdhAddEnglishCounterW(
            query,
            w!("\\GPU Engine(*)\\Utilization Percentage"),
            0,
            &mut counter,
          ) != 0
          || PdhCollectQueryData(query) != 0
        {
          eprintln!("Other programs' use of the GPU cannot be read here");
          return None;
        }
      }
      std::thread::sleep(FIRST_SPAN);
      let mut counters = Self {
        query,
        counter,
        ignored: ignored(),
        read_at: Instant::now(),
        busy: false,
      };
      counters.busy = counters.read().is_some_and(|load| settle(false, load));
      Some(counters)
    }

    /// The busiest other program's share of an engine since the last reading.
    fn read(&mut self) -> Option<f64> {
      if unsafe { PdhCollectQueryData(self.query) } != 0 {
        return None;
      }
      let (mut size, mut count) = (0_u32, 0_u32);
      let asked = unsafe {
        PdhGetFormattedCounterArrayW(self.counter, PDH_FMT_DOUBLE, &mut size, &mut count, None)
      };
      if asked != PDH_MORE_DATA {
        return None;
      }
      // The items, then the names they point to, in one buffer, aligned for
      // the items.
      let mut buffer = vec![0_u64; (size as usize).div_ceil(8)];
      let items = buffer.as_mut_ptr().cast::<PDH_FMT_COUNTERVALUE_ITEM_W>();
      let filled = unsafe {
        PdhGetFormattedCounterArrayW(
          self.counter,
          PDH_FMT_DOUBLE,
          &mut size,
          &mut count,
          Some(items),
        )
      };
      if filled != 0 {
        return None;
      }
      // Safety: PDH wrote `count` items at the start of the buffer, each
      // naming a string inside it.
      let items = unsafe { std::slice::from_raw_parts(items, count as usize) };
      let readings: Vec<(String, f64)> = items
        .iter()
        .filter(|item| {
          matches!(
            item.FmtValue.CStatus,
            PDH_CSTATUS_VALID_DATA | PDH_CSTATUS_NEW_DATA
          )
        })
        .filter_map(|item| {
          let name = unsafe { item.szName.to_string() }.ok()?;
          Some((name, unsafe { item.FmtValue.Anonymous.doubleValue }))
        })
        .collect();
      Some(others_load(
        readings.iter().map(|(name, value)| (name.as_str(), *value)),
        &self.ignored,
      ))
    }
  }

  /// The helper's process, its parent the app, and the desktop window
  /// manager's.
  fn ignored() -> Vec<u32> {
    let own = unsafe { GetCurrentProcessId() };
    let mut ignored = vec![own];
    let Ok(snapshot) = (unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) }) else {
      return ignored;
    };
    let mut entry = PROCESSENTRY32W {
      dwSize: size_of::<PROCESSENTRY32W>() as u32,
      ..Default::default()
    };
    let mut found = unsafe { Process32FirstW(snapshot, &mut entry) }.is_ok();
    while found {
      let length = entry
        .szExeFile
        .iter()
        .position(|&unit| unit == 0)
        .unwrap_or(entry.szExeFile.len());
      if entry.th32ProcessID == own {
        ignored.push(entry.th32ParentProcessID);
      } else if String::from_utf16_lossy(&entry.szExeFile[..length]).eq_ignore_ascii_case("dwm.exe")
      {
        ignored.push(entry.th32ProcessID);
      }
      found = unsafe { Process32NextW(snapshot, &mut entry) }.is_ok();
    }
    let _ = unsafe { CloseHandle(snapshot) };
    ignored
  }
}

#[cfg(test)]
mod tests {
  use super::{others_load, settle};

  #[test]
  fn the_busiest_other_program_counts_not_the_desktop_added_up() {
    let readings = [
      // The desktop's everyday programs on the 3D engine: a quarter together,
      // a little each.
      (
        "pid_40_luid_0x00000000_0x0000C2A1_phys_0_eng_0_engtype_3D",
        18.0,
      ),
      (
        "pid_41_luid_0x00000000_0x0000C2A1_phys_0_eng_0_engtype_3D",
        7.0,
      ),
      // A game, its work on two engines, counted on the busier: NVIDIA names
      // its 3D engine Graphics.
      (
        "pid_50_luid_0x00000000_0x0000C2A1_phys_0_eng_1_engtype_Graphics_1",
        64.0,
      ),
      (
        "pid_50_luid_0x00000000_0x0000C2A1_phys_0_eng_2_engtype_Compute_0",
        9.0,
      ),
      // The helper itself, which does not count.
      (
        "pid_7_luid_0x00000000_0x0000C2A1_phys_0_eng_0_engtype_3D",
        90.0,
      ),
      // A video playing, on the decoder, which models do not use.
      (
        "pid_55_luid_0x00000000_0x0000C2A1_phys_0_eng_3_engtype_VideoDecode",
        80.0,
      ),
      ("not an engine", 99.0),
    ];
    let load = others_load(readings, &[7]);
    assert!((load - 64.0).abs() < 1e-9, "{load}");
    // Without the game, the desktop alone stays quiet.
    let desktop = others_load(readings[..2].iter().copied(), &[7]);
    assert!((desktop - 18.0).abs() < 1e-9, "{desktop}");
  }

  #[test]
  fn the_gpu_stays_busy_until_well_below_where_it_became_busy() {
    assert!(!settle(false, 29.0));
    assert!(settle(false, 30.0));
    // Between the two, it stays as it was.
    assert!(settle(true, 20.0));
    assert!(!settle(false, 20.0));
    assert!(!settle(true, 14.0));
  }
}
