// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Reads the picture of each video given and says how long it took and what
//! it found, writing every reading to `<video>.picture.csv` beside it in the
//! system's temporary folder for a closer look.
//!
//! `cargo run --release -p screenwide-activity --example read_picture -- <video>…`

use std::time::Instant;

use screenwide_activity::{read_picture, states, PictureState, SAMPLES_PER_SECOND};

fn main() {
  for path in std::env::args().skip(1) {
    let started = Instant::now();
    let readings = match read_picture(path.as_ref()) {
      Ok(readings) => readings,
      Err(error) => {
        eprintln!("{path}: {error}");
        continue;
      }
    };
    let took = started.elapsed().as_secs_f64();
    let seconds = readings.len() as f64 / f64::from(SAMPLES_PER_SECOND);
    let found = states(&readings);
    let count = |state| found.iter().filter(|found| **found == state).count();
    let share = |state| 100.0 * count(state) as f64 / found.len().max(1) as f64;
    println!(
      "{path}\n  {seconds:.0} s read in {took:.1} s ({:.0}× real time); still {:.0}%, moving {:.0}%, new {:.1}%",
      seconds / took.max(1e-9),
      share(PictureState::Still),
      share(PictureState::Moving),
      share(PictureState::New),
    );
    let name = std::path::Path::new(&path)
      .file_stem()
      .map_or_else(|| "video".into(), |stem| stem.to_string_lossy());
    let csv = std::env::temp_dir().join(format!("{name}.picture.csv"));
    let rows: String = readings
      .iter()
      .map(|r| format!("{},{:.4},{:.4}\n", r.ms, r.changed, r.novelty))
      .collect();
    if let Err(error) = std::fs::write(&csv, format!("ms,changed,novelty\n{rows}")) {
      eprintln!("  could not write {}: {error}", csv.display());
    }
  }
}
