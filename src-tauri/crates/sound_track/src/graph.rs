// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The pieces as an FFmpeg filter graph, from a moment of the timeline to
//! another: the export reads the whole timeline, the preview from where
//! playback starts. Every piece opens its own file (`amovie`), so the graph
//! needs no inputs of its own and goes into any command as it is.

use std::path::Path;

use crate::pieces::{Curve, Fade, Piece};
use crate::store::singing::PART_LEVEL;

fn seconds(ms: u64) -> String {
  format!("{}.{:03}", ms / 1_000, ms % 1_000)
}

fn curve(curve: Curve) -> &'static str {
  match curve {
    Curve::Linear => "tri",
    Curve::EqualPower => "qsin",
  }
}

fn fade(kind: &str, start_ms: u64, fade: Fade) -> String {
  format!(
    ",afade=t={kind}:st={}:d={}:curve={}",
    seconds(start_ms),
    seconds(fade.ms),
    curve(fade.curve)
  )
}

/// `path` written as a value inside a filter graph. It is escaped twice, as
/// FFmpeg reads it twice: once as the graph, then as the filter's options.
pub fn filter_path(path: &Path) -> String {
  let escape = |text: &str, special: &[char]| {
    let mut escaped = String::with_capacity(text.len());
    for character in text.chars() {
      if special.contains(&character) {
        escaped.push('\\');
      }
      escaped.push(character);
    }
    escaped
  };
  let option = escape(&path.to_string_lossy(), &['\\', '\'', ':']);
  escape(&option, &['\\', '\'', '[', ']', ',', ';'])
}

/// How a clip that makes way comes down while the voice speaks: by about
/// ten decibels under a voice at its usual level, easing back after it.
const MAKE_WAY: &str = "sidechaincompress=threshold=0.015:ratio=3:attack=20:release=600:makeup=1";
/// How the singing in a song that makes way gets out of the voice's way:
/// all the way down while it speaks.
const SINGING_OUT: &str =
  "sidechaincompress=threshold=0.015:ratio=20:attack=10:release=400:makeup=1";

/// Whether any of `pieces` comes down while the voice speaks, so the caller
/// hands the graph the voice.
pub fn makes_way(pieces: &[Piece]) -> bool {
  pieces.iter().any(|piece| piece.makes_way)
}

/// The filters that read `path` as `piece` plays it from `from_ms` to
/// `to_ms` of the timeline, the first moment heard at the start of the
/// output: the piece's own file, or the singing separated from it.
fn chain(piece: &Piece, path: &Path, (from_ms, to_ms): (u64, u64)) -> String {
  // Where the window starts inside a fade, the piece is read from its
  // start so the fade keeps its shape; elsewhere, from the window, so a
  // preview started late in a long song does not read all of it first.
  let reads_from = if from_ms <= piece.start_ms
    || from_ms < piece.start_ms + piece.fade_in.ms
    || from_ms > piece.end_ms.saturating_sub(piece.fade_out.ms)
  {
    piece.start_ms
  } else {
    from_ms
  };
  let seek = piece.file_start_ms + (reads_from - piece.start_ms);
  let length = piece.end_ms - reads_from;
  let mut chain = format!(
    "amovie={}:seek_point={seek_at},atrim=start={seek_at}:duration={},asetpts=PTS-STARTPTS",
    filter_path(path),
    seconds(length),
    seek_at = seconds(seek),
  );
  if piece.fade_in.ms > 0 && reads_from == piece.start_ms {
    chain.push_str(&fade("in", 0, piece.fade_in));
  }
  if piece.fade_out.ms > 0 {
    chain.push_str(&fade(
      "out",
      piece.end_ms - piece.fade_out.ms - reads_from,
      piece.fade_out,
    ));
  }
  let heard_from = from_ms.max(piece.start_ms) - reads_from;
  let heard_to = to_ms.min(piece.end_ms) - reads_from;
  if heard_from > 0 || heard_to < length {
    chain.push_str(&format!(
      ",atrim=start={}:end={},asetpts=PTS-STARTPTS",
      seconds(heard_from),
      seconds(heard_to)
    ));
  }
  if piece.gain_db != 0.0 {
    chain.push_str(&format!(",volume={:.2}dB", piece.gain_db));
  }
  let delay = from_ms.max(piece.start_ms) - from_ms;
  if delay > 0 {
    chain.push_str(&format!(",adelay=delays={delay}:all=1"));
  }
  chain
}

/// `labels` summed into `[out]`. `normalize=0`, as for the recorded tracks:
/// a sound effect starting must not make the song under it quieter.
fn mixed(labels: &[String], then: &str, out: &str) -> String {
  format!(
    "{}amix=inputs={}:normalize=0:dropout_transition=0{then}[{out}]",
    labels.concat(),
    labels.len()
  )
}

/// The filters that play `pieces` from `from_ms` to `to_ms` of the
/// timeline, the first moment heard at the start of the output, mixed into
/// `[out]`, without a closing `;`. `then` follows the mix, such as a
/// resample to the output's format. Given the voice, as a label the graph
/// may read once, the pieces that make way come down while it speaks, and
/// the singing separated from them gets out of its way entirely. `None`
/// where nothing is heard then.
pub fn filter(
  pieces: &[Piece],
  window: (u64, u64),
  out: &str,
  then: &str,
  voice: Option<&str>,
) -> Option<String> {
  let (from_ms, to_ms) = window;
  let mut graph = String::new();
  // As heard, those making way, and the singing in those making way.
  let mut buses: [Vec<String>; 3] = Default::default();
  for (index, piece) in pieces
    .iter()
    .filter(|piece| piece.start_ms < to_ms && piece.end_ms > from_ms)
    .enumerate()
  {
    let label = format!("[sound{index}]");
    let ducked = voice.is_some() && piece.makes_way;
    let rest_bus = usize::from(ducked);
    match &piece.parts {
      // Separated, the song plays as its two parts: the rest, and the
      // singing, which gets out of the voice's way entirely, or is left out
      // of an instrumental song.
      Some(parts) if ducked || piece.instrumental => {
        // The parts are kept quieter than the song, to fit their files.
        let made_up = Piece {
          gain_db: piece.gain_db - 20.0 * f64::from(PART_LEVEL).log10(),
          ..piece.clone()
        };
        graph.push_str(&format!(
          "{}[rest{index}];",
          chain(&made_up, &parts.rest, window)
        ));
        buses[rest_bus].push(format!("[rest{index}]"));
        if !piece.instrumental {
          graph.push_str(&format!(
            "{}[sung{index}];",
            chain(&made_up, &parts.singing, window)
          ));
          buses[2].push(format!("[sung{index}]"));
        }
      }
      _ => {
        graph.push_str(&format!("{}{label};", chain(piece, &piece.path, window)));
        buses[rest_bus].push(label);
      }
    }
  }
  if buses.iter().all(Vec::is_empty) {
    return None;
  }
  let mut heard = Vec::new();
  if !buses[0].is_empty() {
    graph.push_str(&mixed(&buses[0], "", "soundplain"));
    graph.push(';');
    heard.push("[soundplain]".to_owned());
  }
  let ducking: Vec<(&Vec<String>, &str, &str)> = [
    (&buses[1], "soundway", MAKE_WAY),
    (&buses[2], "soundsung", SINGING_OUT),
  ]
  .into_iter()
  .filter(|(bus, _, _)| !bus.is_empty())
  .collect();
  match voice {
    Some(voice) if !ducking.is_empty() => {
      let voices: Vec<String> = if ducking.len() == 1 {
        vec![voice.to_owned()]
      } else {
        graph.push_str(&format!("{voice}asplit=2[soundvoice0][soundvoice1];"));
        vec!["[soundvoice0]".to_owned(), "[soundvoice1]".to_owned()]
      };
      for ((bus, name, comes_down), voice) in ducking.into_iter().zip(voices) {
        graph.push_str(&mixed(bus, "", name));
        graph.push_str(&format!(";[{name}]{voice}{comes_down}[{name}down];"));
        heard.push(format!("[{name}down]"));
      }
    }
    // Handed in but with nothing making way from here on, the voice still
    // has to go somewhere in the graph.
    Some(voice) => graph.push_str(&format!("{voice}anullsink;")),
    None => {}
  }
  graph.push_str(&mixed(&heard, then, out));
  Some(graph)
}

#[cfg(test)]
mod tests;
