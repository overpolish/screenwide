// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The lines a shape can land on in one axis, and which of them it takes.

/// One line a position can land on, in source pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct AxisGuide {
  pub(crate) position: f64,
  /// Another annotation's line rather than one of the canvas's own, which
  /// wins ties and draws in the object colour.
  pub(crate) object: bool,
}

/// The shortest move that lands one of `lines` on a guide, and the guide it
/// landed on.
///
/// Every line the moving shape offers is measured against every guide and the
/// smallest move wins, so a box aligns by whichever of its edges or its centre
/// is nearest a candidate. The move is returned rather than the position,
/// because the whole subject travels by it. An object guide beats a canvas one
/// at the same distance, exactly as the layer engines resolve a tie.
pub(crate) fn snap_axis(
  lines: &[f64],
  guides: &[AxisGuide],
  threshold: f64,
) -> Option<(f64, AxisGuide)> {
  let mut best: Option<(f64, AxisGuide)> = None;
  for line in lines {
    for guide in guides {
      let offset = guide.position - line;
      let distance = offset.abs();
      if distance > threshold {
        continue;
      }
      let wins = match best {
        None => true,
        Some((chosen, chosen_guide)) => {
          distance < chosen.abs()
            || (distance == chosen.abs() && guide.object && !chosen_guide.object)
        }
      };
      if wins {
        best = Some((offset, *guide));
      }
    }
  }
  best
}

/// What one axis ended up taking.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum AxisWinner {
  /// A line of the canvas or of another annotation.
  Guide(AxisGuide),
  /// A detected element's edge, under the shape's point subject.
  Edge,
  /// Equal spacing with a pair of other annotations.
  Gap,
}

impl AxisWinner {
  pub(crate) fn guide(self) -> Option<AxisGuide> {
    match self {
      Self::Guide(guide) => Some(guide),
      _ => None,
    }
  }

  pub(crate) fn is_edge(self) -> bool {
    matches!(self, Self::Edge)
  }

  pub(crate) fn is_gap(self) -> bool {
    matches!(self, Self::Gap)
  }

  /// How much a candidate is worth when two of them ask for the same move:
  /// the closer it is to something the hand placed deliberately, the lower
  /// this is. Another annotation's line first, then an element's edge, then
  /// equal spacing, and the canvas's own lines last.
  fn rank(self) -> u8 {
    match self {
      Self::Guide(guide) if guide.object => 0,
      Self::Edge => 1,
      Self::Gap => 2,
      Self::Guide(_) => 3,
    }
  }
}

/// Which of one axis's candidates a sample takes: a guide one of the moving
/// shape's own lines reaches, an element edge its point subject reaches, or
/// equal spacing with a pair of other annotations. The move, and what asked
/// for it.
///
/// Nearest wins; [`AxisWinner::rank`] breaks an exact tie.
pub(crate) fn resolve_axis(
  guide: Option<(f64, AxisGuide)>,
  edge: Option<f64>,
  gap: Option<f64>,
) -> (f64, Option<AxisWinner>) {
  let mut best: Option<(f64, AxisWinner)> = None;
  let mut offer = |offset: f64, winner: AxisWinner| {
    let wins = match best {
      None => true,
      Some((chosen, taken)) => {
        offset.abs() < chosen.abs()
          || (offset.abs() == chosen.abs() && winner.rank() < taken.rank())
      }
    };
    if wins {
      best = Some((offset, winner));
    }
  };
  if let Some((offset, guide)) = guide {
    offer(offset, AxisWinner::Guide(guide));
  }
  if let Some(offset) = edge {
    offer(offset, AxisWinner::Edge);
  }
  if let Some(offset) = gap {
    offer(offset, AxisWinner::Gap);
  }
  best.map_or((0.0, None), |(offset, winner)| (offset, Some(winner)))
}
