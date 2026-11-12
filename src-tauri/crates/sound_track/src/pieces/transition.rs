// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! How songs that touch are joined. Where both have a steady beat and
//! their tempos are close, the earlier comes to the later's tempo and the
//! two blend band by band on the beat (`blend`), as a DJ mixes; otherwise,
//! or where there is too little of either to blend over, they crossfade.
//! Two voices singing at once sound worst of all, so where the later song
//! has singing, the earlier's is left out as it goes, wherever it is
//! separated.

mod blend;

use super::{Fade, Laid, Piece, CROSSFADE_MS, TOUCHING_MS};
use crate::clip::AudioClipKind;

/// How long two pieces crossfade where one hands over to the other on a
/// beat, in ms: short, so the handover lands on the beat, but not so short
/// that it clicks.
const SWAP_MS: u64 = 20;

/// Joins every pair of songs in `laid` that touch.
pub(super) fn join_songs(laid: &mut [Laid<'_>]) {
  let mut songs: Vec<usize> = (0..laid.len())
    .filter(|&index| laid[index].clip.kind == AudioClipKind::Music)
    .collect();
  songs.sort_by_key(|&index| laid[index].start_ms);
  for pair in songs.windows(2) {
    let Ok([earlier, later]) = laid.get_disjoint_mut([pair[0], pair[1]]) else {
      continue;
    };
    let join = later.start_ms;
    let heard_to = earlier.pieces.last().map_or(0, |piece| piece.end_ms);
    if earlier.end_ms.abs_diff(join) > TOUCHING_MS || heard_to != earlier.end_ms {
      continue;
    }
    join_pair(earlier, later, join);
  }
}

fn join_pair(earlier: &mut Laid<'_>, later: &mut Laid<'_>, join: u64) {
  // Only where the earlier song's singing is separated is there any to
  // leave out.
  let drops_singing = !later.clip.is_instrumental && earlier.file.separated.is_some();
  let joined = match (earlier.file.music.beat, later.file.music.beat) {
    (Some(ours), Some(theirs)) if ours.matches(&theirs) => {
      blend::blend(earlier, later, join, (ours, theirs), drops_singing)
    }
    _ => false,
  };
  if !joined {
    crossfade(earlier, later, join, drops_singing);
  }
}

/// The earlier song plays on past the join and the later starts before it,
/// each by as much as its file holds, up to half the crossfade; a side that
/// holds less leaves the rest to the other, and where neither holds enough
/// the crossfade happens inside the clips. The crossfade takes the place of
/// the two clips' own fades at the join.
fn crossfade(earlier: &mut Laid<'_>, later: &mut Laid<'_>, join: u64, drops_singing: bool) {
  let length = |laid: &Laid<'_>| laid.end_ms - laid.start_ms;
  let crossfade = CROSSFADE_MS.min(length(earlier) / 2).min(length(later) / 2);
  let after = earlier.pieces.last().map_or(0, |piece| {
    earlier
      .file
      .duration_ms
      .saturating_sub(piece.file_start_ms + piece.length_ms())
  });
  let before = later.pieces.first().map_or(0, |piece| piece.file_start_ms);
  let mut early = (crossfade / 2).min(before);
  let late = (crossfade - early).min(after);
  early = (crossfade - late).min(before);
  if let Some(last) = earlier.pieces.last_mut() {
    last.end_ms = join + late;
    last.fade_out = Fade::equal_power(crossfade);
    let from = join - early;
    if drops_singing && from >= last.start_ms + SWAP_MS {
      let mut going = split(last, from);
      going.instrumental = true;
      earlier.added.push(going);
    }
  }
  if let Some(first) = later.pieces.first_mut() {
    first.start_ms = join - early;
    first.file_start_ms -= early;
    first.fade_in = Fade::equal_power(crossfade);
  }
}

/// The rest of `piece` from `at_ms`, as a piece of its own that keeps its
/// fade out, `piece` ending just after: the two hand over around `at_ms`,
/// straight across, as both play the same sound there. The piece must start
/// at least half a swap before `at_ms`.
fn split(piece: &mut Piece, at_ms: u64) -> Piece {
  let half = SWAP_MS / 2;
  let mut rest = piece.clone();
  rest.start_ms = at_ms - half;
  rest.file_start_ms = piece.file_start_ms + (rest.start_ms - piece.start_ms);
  rest.fade_in = Fade::linear(SWAP_MS);
  piece.end_ms = at_ms + half;
  piece.fade_out = Fade::linear(SWAP_MS);
  rest
}

#[cfg(test)]
mod tests;
