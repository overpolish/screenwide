// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Room made for the sound effects in the songs and the system audio, band
//! by band: each is split into four bands, as are the effects, and each band
//! comes down only as far as the effects have sound in that same band at
//! that moment, so a deep whoosh carves the lows and a click the highs. Only
//! part of each band is carved, so a band never drops by more than a few
//! decibels, the bass least of all, and the music never pumps.

use super::mixed;

/// The bands, as Linkwitz-Riley crossovers (two second-order filters each,
/// so the four add back up to the whole), and how much of each is carved:
/// the share of the band the compressor's output takes, the rest left as it
/// was, which holds the dip to about 6 dB, and to about 3 dB in the bass.
const BANDS: [(&str, f64); 4] = [
  ("lowpass=f=200,lowpass=f=200", 0.3),
  (
    "highpass=f=200,highpass=f=200,lowpass=f=1000,lowpass=f=1000",
    0.5,
  ),
  (
    "highpass=f=1000,highpass=f=1000,lowpass=f=4000,lowpass=f=4000",
    0.5,
  ),
  ("highpass=f=4000,highpass=f=4000", 0.5),
];

/// How a band comes down under the effects' same band: quickly, so the
/// start of a hit is clear, and back over a fifth of a second after it.
const CARVE: &str = "sidechaincompress=threshold=0.02:ratio=8:attack=5:release=200:makeup=1";

/// The filters that carve `stream` by `key`, both labels the graph reads
/// once, into `[{name}]`, which it returns, each filter ending in `;`.
pub(super) fn carve(graph: &mut String, (stream, key): (&str, &str), name: &str) -> String {
  let labels = |part: &str| -> String {
    (0..BANDS.len())
      .map(|band| format!("[{name}{part}{band}]"))
      .collect()
  };
  graph.push_str(&format!("{stream}asplit={}{};", BANDS.len(), labels("s")));
  // The compressor stops where either input does, so the effects go on as
  // silence past their last sound, and the stream carved plays to its end.
  graph.push_str(&format!("{key}apad,asplit={}{};", BANDS.len(), labels("k")));
  for (band, (filters, share)) in BANDS.iter().enumerate() {
    graph.push_str(&format!(
      "[{name}s{band}]{filters}[{name}b{band}];\
       [{name}k{band}]{filters}[{name}kb{band}];\
       [{name}b{band}][{name}kb{band}]{CARVE}:mix={share}[{name}c{band}];"
    ));
  }
  graph.push_str(&format!(
    "{}amix=inputs={}:normalize=0[{name}];",
    labels("c"),
    BANDS.len()
  ));
  format!("[{name}]")
}

/// Mixes the sound effects `effects` among the heard `songs`, and carves the
/// songs and `beside` for them, each by the effects as heard.
pub(super) fn make_room(
  graph: &mut String,
  effects: &[String],
  songs: &mut Vec<String>,
  beside: &mut [String],
) {
  graph.push_str(&mixed(effects, "", "soundfx"));
  graph.push(';');
  let carves_songs = !songs.is_empty();
  let keys = usize::from(carves_songs) + beside.len();
  if keys == 0 {
    songs.push("[soundfx]".to_owned());
    return;
  }
  let key = |index: usize| format!("[soundkey{index}]");
  graph.push_str(&format!(
    "[soundfx]asplit={}[soundfxheard]{};",
    keys + 1,
    (0..keys).map(key).collect::<String>()
  ));
  let mut next = 0;
  if carves_songs {
    graph.push_str(&mixed(songs, "", "soundsongs"));
    graph.push(';');
    *songs = vec![carve(
      graph,
      ("[soundsongs]", &key(next)),
      "soundsongscarved",
    )];
    next += 1;
  }
  for (index, label) in beside.iter_mut().enumerate() {
    *label = carve(graph, (label, &key(next)), &format!("soundbeside{index}"));
    next += 1;
  }
  songs.push("[soundfxheard]".to_owned());
}
