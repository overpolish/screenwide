// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Which picture a mesh background paints.
//!
//! The app's own blob mesh was the only one for a while, so the background is
//! still called a mesh. It now names a generator alongside its colours: the
//! original, which alone reads the blobs and the warp, and eight ported from
//! the reference shader library, which read their colours, the seed, and the
//! canvas seconds. The numbers are a wire format shared with three shading
//! languages, so an id keeps its number for good and new ones are appended.
//!
//! # How fast each generator drifts
//!
//! Every backend hands a generator the same seconds the classic mesh already
//! drifts with: the Metal canvas's `seconds`, the D3D canvas's `motion.x`, and
//! the `time` word of the wgpu mesh uniform. The dispatch that switches on the
//! id scales those seconds once by the generator's `speed` before the
//! generator sees them, because the reference shaders assume `iTime` at full
//! speed and a background behind a recording wants a drift, not a demo. The
//! shaders carry no factor of their own; this table is the `speed` field, and
//! the numbers are tuned by eye against the classic mesh, which is the
//! reference for "subtle".
//!
//! | Generator  | `speed` | Where the seconds enter                         |
//! | ---------- | ------- | ----------------------------------------------- |
//! | `mesh`     | 1.00    | the classic blob drift, unscaled                |
//! | `silk`     | 0.025   | the four phases of a sheet's fold               |
//! | `aurora`   | 0.025   | the accumulator's starting value                |
//! | `fluid`    | 0.025   | the third axis of every fbm lookup              |
//! | `gentle`   | 0.075   | the two phases of the swirl angle               |
//! | `currents` | 0.025   | the turbulence axis and the warp loop's phases  |
//! | `paint`    | 0.025   | every octave's phase                            |
//! | `ribbons`  | 0.025   | the flow offset and the two ripple phases       |
//! | `strata`   | 0.025   | the tectonic warp's domain, compression, shear  |
//!
//! Zero is a still, which is what a thumbnail and a screenshot export ask
//! for. There the seed shift alone tells two pictures of one generator apart.

/// One generator: the name settings carry, the number every shader switches
/// on, how many colours its palette takes, and how fast it drifts.
pub(crate) struct MeshGenerator {
  pub(crate) color_count: usize,
  pub(crate) id: u32,
  pub(crate) name: &'static str,
  /// What the canvas seconds are multiplied by before the generator reads
  /// them, tabled above. The dispatch applies it once, so a shader never
  /// carries a factor of its own.
  pub(crate) speed: f32,
}

const GENERATORS: [MeshGenerator; 9] = [
  MeshGenerator {
    color_count: 4,
    id: 0,
    name: "mesh",
    speed: 1.0,
  },
  MeshGenerator {
    color_count: 4,
    id: 1,
    name: "silk",
    speed: 0.025,
  },
  MeshGenerator {
    color_count: 3,
    id: 2,
    name: "aurora",
    speed: 0.025,
  },
  MeshGenerator {
    color_count: 4,
    id: 3,
    name: "fluid",
    speed: 0.025,
  },
  MeshGenerator {
    color_count: 3,
    id: 4,
    name: "gentle",
    speed: 0.075,
  },
  MeshGenerator {
    color_count: 4,
    id: 5,
    name: "currents",
    speed: 0.025,
  },
  MeshGenerator {
    color_count: 4,
    id: 6,
    name: "paint",
    speed: 0.025,
  },
  MeshGenerator {
    color_count: 4,
    id: 7,
    name: "ribbons",
    speed: 0.025,
  },
  MeshGenerator {
    color_count: 4,
    id: 8,
    name: "strata",
    speed: 0.025,
  },
];

/// The longest palette any generator reads, and the width of the colour
/// array every backend hands them.
pub(crate) const MAXIMUM_GENERATOR_COLORS: usize = 4;

/// The name settings written before generators existed carry, and the one an
/// absent field falls back to.
pub(crate) const DEFAULT_GENERATOR: &str = "mesh";

pub(crate) fn default_generator() -> String {
  DEFAULT_GENERATOR.to_owned()
}

/// Resolves a generator seed on the CPU so every D3D11 draw receives
/// stable domain coordinates instead of recalculating a large GPU sine hash.
#[cfg(any(target_os = "windows", test))]
pub(crate) fn generator_seed_shift(seed: u32) -> [f32; 3] {
  // Cache the wire-format f32 result so later preview/export calls on threads
  // with a different rounding mode receive the exact same domain shift.
  use std::collections::HashMap;
  static SHIFTS: std::sync::OnceLock<std::sync::Mutex<HashMap<u32, [f32; 3]>>> =
    std::sync::OnceLock::new();
  let shifts = SHIFTS.get_or_init(|| std::sync::Mutex::new(HashMap::new()));
  let mut shifts = shifts.lock().unwrap();
  if let Some(shift) = shifts.get(&seed) {
    return *shift;
  }
  let value = f64::from(seed);
  let fract = |value: f64| value - value.floor();
  let shift = [
    ((fract((value * 12.9898 + 4.1).sin() * 43_758.5453) - 0.5) * 1.5) as f32,
    ((fract((value * 78.233 + 1.7).sin() * 43_758.5453) - 0.5) * 1.5) as f32,
    (fract((value * 39.425 + 9.3).sin() * 43_758.5453) * std::f64::consts::TAU) as f32,
  ];
  shifts.insert(seed, shift);
  shift
}

/// The generator a name asks for. `None` for a name this build does not know,
/// which the caller reports rather than painting something else.
pub(crate) fn mesh_generator(name: &str) -> Option<&'static MeshGenerator> {
  let name = if name.is_empty() {
    DEFAULT_GENERATOR
  } else {
    name
  };
  GENERATORS.iter().find(|generator| generator.name == name)
}

/// A ported generator's palette as the shaders read it: four opaque colours
/// off the front of the settings, with the last repeated to fill a palette
/// shorter than the shader has room for rather than leaving it black.
///
/// The original mesh is laid out the other way around, one colour per blob
/// with the base behind them, so this is only for the ported generators.
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
pub(crate) fn generator_palette(colors: &[String]) -> Result<[[f32; 4]; 4], String> {
  let last = colors.last().map_or(Ok([0, 0, 0, u8::MAX]), |value| {
    super::parse_hex_colour(value)
  })?;
  let mut palette = [[0.0; 4]; 4];
  for (index, slot) in palette.iter_mut().enumerate() {
    let colour = match colors.get(index) {
      Some(value) => super::parse_hex_colour(value)?,
      None => last,
    };
    let [red, green, blue, _] = colour.map(|channel| f32::from(channel) / 255.0);
    *slot = [red, green, blue, 1.0];
  }
  Ok(palette)
}

/// The colours as the native canvas shaders read them.
///
/// The app's own mesh takes one colour per blob with the base behind them,
/// which is how the settings carry them, so they are handed over as they are.
/// A ported generator reads its palette off the front instead.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
pub(crate) fn canvas_colors(
  generator: &MeshGenerator,
  colors: &[String],
) -> Result<[[f32; 4]; 5], String> {
  let mut canvas = [[0.0, 0.0, 0.0, 1.0]; 5];
  if generator.id == 0 {
    for (index, value) in colors.iter().take(5).enumerate() {
      let [red, green, blue, _] = super::parse_hex_colour(value)?.map(|c| f32::from(c) / 255.0);
      canvas[index] = [red, green, blue, 1.0];
    }
    return Ok(canvas);
  }
  canvas[..4].copy_from_slice(&generator_palette(colors)?);
  Ok(canvas)
}

#[cfg(test)]
pub(crate) fn every_generator() -> &'static [MeshGenerator] {
  &GENERATORS
}

#[cfg(test)]
#[path = "mesh_generator/tests.rs"]
mod tests;
