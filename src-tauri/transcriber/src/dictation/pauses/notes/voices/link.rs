// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The speakers of overlapping windows of a call joined into the call's
//! speakers. Each window's diarization tells its own voices apart, numbered
//! afresh; here a voice heard in two windows becomes one speaker. Windows
//! that overlap agree about who spoke in the overlap, and that ties two
//! windows' speakers together first; every other pair is joined by the voices'
//! fingerprints, most alike first, while they stay alike enough. Two voices
//! of one window are never joined: its diarization already told them apart.

/// A window's speaker: which window, and the fingerprint of their voice
/// there, with how much of their speech it was taken from, in milliseconds.
pub struct Local {
  pub window: usize,
  pub fingerprint: Option<Vec<f32>>,
  pub heard_ms: u64,
}

/// How much two windows' speakers must agree in their overlap to be one
/// speaker: each must spend at least this share of their talking there
/// together.
const AGREEMENT: f32 = 0.5;
/// The least time two speakers talk together in an overlap to be tied by
/// it, in milliseconds.
const AGREEMENT_MIN_MS: u64 = 2_000;

/// Joins `locals` into speakers: `ties` are pairs the overlaps agree on,
/// with the time they talked together in milliseconds, and each local's
/// talking in that overlap; `threshold` is the least cosine similarity two
/// groups' fingerprints may have to be joined. Returns each local's speaker,
/// numbered from 0 in the order of `locals`.
pub fn link(locals: &[Local], ties: &[Tie], threshold: f32) -> Vec<usize> {
  let n = locals.len();
  let mut group: Vec<usize> = (0..n).collect();
  let find = |group: &mut Vec<usize>, mut at: usize| {
    while group[at] != at {
      group[at] = group[group[at]];
      at = group[at];
    }
    at
  };
  let windows_of = |group: &mut Vec<usize>, root: usize| -> Vec<usize> {
    (0..n)
      .filter(|&i| find(group, i) == root)
      .map(|i| locals[i].window)
      .collect()
  };

  // The overlaps' agreements, the strongest first.
  let mut agreed: Vec<&Tie> = ties
    .iter()
    .filter(|tie| {
      tie.together_ms >= AGREEMENT_MIN_MS
        && tie.together_ms as f32 >= AGREEMENT * tie.a_ms as f32
        && tie.together_ms as f32 >= AGREEMENT * tie.b_ms as f32
    })
    .collect();
  agreed.sort_by_key(|tie| std::cmp::Reverse(tie.together_ms));
  for tie in agreed {
    join(&mut group, tie.a, tie.b, &find, &windows_of);
  }

  // Then the fingerprints: average linkage over each group's voices,
  // weighted by how much speech each was taken from.
  loop {
    let roots: Vec<usize> = {
      let mut roots: Vec<usize> = (0..n).map(|i| find(&mut group, i)).collect();
      roots.sort_unstable();
      roots.dedup();
      roots
    };
    let centroid = |group: &mut Vec<usize>, root: usize| -> Option<Vec<f32>> {
      let mut sum: Option<Vec<f32>> = None;
      for i in (0..n).filter(|&i| find(group, i) == root) {
        let Some(fingerprint) = &locals[i].fingerprint else {
          continue;
        };
        let weight = locals[i].heard_ms.max(1) as f32;
        let sum = sum.get_or_insert_with(|| vec![0.0; fingerprint.len()]);
        sum
          .iter_mut()
          .zip(fingerprint)
          .for_each(|(total, value)| *total += weight * value);
      }
      sum.map(|mut sum| {
        let length = sum.iter().map(|value| value * value).sum::<f32>().sqrt();
        sum
          .iter_mut()
          .for_each(|value| *value /= length.max(f32::EPSILON));
        sum
      })
    };
    let centroids: Vec<(usize, Vec<f32>, Vec<usize>)> = roots
      .iter()
      .filter_map(|&root| {
        let windows = windows_of(&mut group, root);
        centroid(&mut group, root).map(|c| (root, c, windows))
      })
      .collect();
    let mut best: Option<(f32, usize, usize)> = None;
    for (i, (a, ca, wa)) in centroids.iter().enumerate() {
      for (b, cb, wb) in &centroids[i + 1..] {
        if wa.iter().any(|window| wb.contains(window)) {
          continue;
        }
        let similarity: f32 = ca.iter().zip(cb).map(|(x, y)| x * y).sum();
        if similarity >= threshold && best.is_none_or(|(s, _, _)| similarity > s) {
          best = Some((similarity, *a, *b));
        }
      }
    }
    let Some((_, a, b)) = best else {
      break;
    };
    group[b] = a;
  }

  // Number the groups in the order their first voice comes in `locals`.
  let mut numbers: Vec<Option<usize>> = vec![None; n];
  let mut next = 0;
  (0..n)
    .map(|i| {
      let root = find(&mut group, i);
      *numbers[root].get_or_insert_with(|| {
        next += 1;
        next - 1
      })
    })
    .collect()
}

/// Two windows' speakers who talked in the same stretch of their overlap.
pub struct Tie {
  pub a: usize,
  pub b: usize,
  pub together_ms: u64,
  /// Each one's talking in the overlap, in milliseconds.
  pub a_ms: u64,
  pub b_ms: u64,
}

fn join(
  group: &mut Vec<usize>,
  a: usize,
  b: usize,
  find: &impl Fn(&mut Vec<usize>, usize) -> usize,
  windows_of: &impl Fn(&mut Vec<usize>, usize) -> Vec<usize>,
) {
  let (ra, rb) = (find(group, a), find(group, b));
  if ra == rb {
    return;
  }
  let wa = windows_of(group, ra);
  if windows_of(group, rb)
    .iter()
    .any(|window| wa.contains(window))
  {
    return;
  }
  group[rb] = ra;
}

#[cfg(test)]
mod tests {
  use super::*;

  fn local(window: usize, fingerprint: &[f32]) -> Local {
    Local {
      window,
      fingerprint: Some(fingerprint.to_vec()),
      heard_ms: 10_000,
    }
  }

  #[test]
  fn alike_voices_in_different_windows_are_one_speaker() {
    let locals = [
      local(0, &[1.0, 0.0]),
      local(0, &[0.0, 1.0]),
      local(1, &[0.96, 0.28]),
      local(1, &[0.6, 0.8]),
    ];
    assert_eq!(link(&locals, &[], 0.9), vec![0, 1, 0, 2]);
  }

  #[test]
  fn two_voices_of_one_window_stay_apart_however_alike() {
    let locals = [local(0, &[1.0, 0.0]), local(0, &[1.0, 0.0])];
    assert_eq!(link(&locals, &[], 0.5), vec![0, 1]);
  }

  #[test]
  fn an_overlap_both_windows_agree_on_joins_unalike_fingerprints() {
    let locals = [local(0, &[1.0, 0.0]), local(1, &[0.0, 1.0])];
    let tie = Tie {
      a: 0,
      b: 1,
      together_ms: 20_000,
      a_ms: 25_000,
      b_ms: 22_000,
    };
    assert_eq!(link(&locals, &[tie], 0.9), vec![0, 0]);
  }
}
