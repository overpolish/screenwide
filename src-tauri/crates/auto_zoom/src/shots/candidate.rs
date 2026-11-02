// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! One way to zoom into a run of beats, when it may start and end, and what
//! it and the move to the next one are worth.

use super::super::beats::Beat;
use super::super::signals::Signals;
use super::super::Area;
use super::pointer::{away, outside_seconds};

/// How long a scene takes to arrive or leave; the twin of `TRANSITION_MS` in
/// `scenes/arrange.rs`.
const TRANSITION_MS: u64 = 600;
/// How long before its first click a shot starts arriving, so it has
/// settled a moment before the click lands.
const LEAD_MS: u64 = 850;
/// How long a shot holds on after its last beat, its leaving included, and
/// the least it may when the next beat is close.
const TAIL_MS: u64 = 1_500;
const MIN_TAIL_MS: u64 = 900;
/// The shortest shot, its arrival and leaving included.
const MIN_CLIP_MS: u64 = 3_500;
/// How long a shot holds on after its last beat when it hands straight over
/// to the next: the handover starts from the work rather than from the whole
/// screen, so it needs no leaving of its own.
const HANDOVER_HOLD_MS: u64 = 300;
/// The shortest shot that hands over to the next.
const MIN_HANDOVER_CLIP_MS: u64 = 2_500;
/// The shortest the whole screen may show between two shots. Two shots
/// closer than this pan from one to the other instead.
pub(super) const MIN_OVERVIEW_MS: u64 = 2_000;
/// Shorter than this, the whole screen showing between two shots reads as
/// the camera pumping.
const CALM_OVERVIEW_MS: u64 = 4_000;
/// What each second a shot arrives after its work began costs. Only a shot
/// held back by the whole screen showing before it, as after a switch to
/// another app, arrives late.
const LATE_COST: f64 = 0.2;

/// How much of the frame the work fills at most.
const FILL: f64 = 0.7;
/// The furthest an auto zoom goes, and the least that is worth a move.
const MAX_AUTO_ZOOM: f64 = 2.0;
const MIN_USEFUL_ZOOM: f64 = 1.25;

/// What a move costs, against a beat of weight one zoomed twice over earning
/// one.
const TRANSITION_COST: f64 = 0.35;
/// What a pan costs for each frame's width it travels.
const PAN_COST: f64 = 0.3;
/// A pan shorter than this, in frames, reads as a twitch.
const MICRO_MOVE: f64 = 0.25;
const MICRO_COST: f64 = 0.4;
/// What the whole screen showing only briefly between two shots costs.
const PUMP_COST: f64 = 0.3;
/// What each second the pointer spends outside a shot's frame costs.
const COVER_COST: f64 = 0.25;
/// The longest the pointer may be away from a shot's frame at a stretch while
/// the shot holds on. The pointer is where the attention is: away for longer,
/// the camera goes with it rather than waiting on the work it left.
const MAX_AWAY_MS: u64 = 1_500;
/// How long the pointer must stay out of a shot's frame after its last beat
/// for the shot to leave with it. Shorter is a flick.
const LEFT_MS: u64 = 300;
/// How far inside a typing shot's frame other work must sit to share it, as
/// a share of the frame.
const TYPING_SHARE_INSET: f64 = 0.1;

#[derive(Clone, Copy, Debug)]
pub(super) struct Candidate {
  /// When it starts arriving, given the room, the latest it may to have
  /// arrived before its work begins, and the latest it may at all.
  pub start_ideal: u64,
  pub start_latest: u64,
  pub start_late: u64,
  /// When it finishes leaving, given the room, and the earliest it may.
  pub end_ideal: u64,
  pub end_earliest: u64,
  /// The earliest it may hand straight over to the next shot.
  pub handover_earliest: u64,
  pub focus: (f64, f64),
  pub zoom: f64,
  pub score: f64,
  /// What it shows, and when the pointer left that after its last beat.
  frame: Area,
  left_ms: Option<u64>,
}

/// The best zoom into `beats` together, or `None` where they are spread too
/// far for a zoom to be worth it. A shot that cannot arrive before
/// `not_before`, because the whole screen shows the beat before it until
/// then, holds on from that late arrival for as long as the shortest shot.
pub(super) fn candidate(
  beats: &[Beat],
  signals: &Signals,
  duration_ms: u64,
  not_before: u64,
) -> Option<Candidate> {
  let shares_a_brought_forward_press =
    beats.len() > 1 && beats.iter().any(|beat| beat.brought_forward);
  if shares_a_brought_forward_press || beats.iter().any(|beat| beat.is_switch) {
    return None;
  }
  // Typing is framed on where it goes, never a middle between it and other
  // work: the text runs on from there, so it would be cut off. Other work
  // shares the shot only well inside that frame; anywhere else, it is a shot
  // of its own that the camera pans or zooms out to.
  let typed = beats
    .iter()
    .filter(|beat| beat.typed)
    .map(|beat| beat.area)
    .reduce(Area::union);
  let area = match typed {
    Some(area) => area,
    None => beats.iter().map(|beat| beat.area).reduce(Area::union)?,
  };
  let zoom = (FILL / area.span().max(FILL / MAX_AUTO_ZOOM)).min(MAX_AUTO_ZOOM);
  if zoom < MIN_USEFUL_ZOOM {
    return None;
  }
  // The frame shows the same share of the picture's width and height.
  let half = 0.5 / zoom;
  let (x, y) = area.centre();
  let focus = (x.clamp(half, 1.0 - half), y.clamp(half, 1.0 - half));
  let frame = Area {
    left: focus.0 - half,
    top: focus.1 - half,
    right: focus.0 + half,
    bottom: focus.1 + half,
  };
  let inset = 2.0 * half * TYPING_SHARE_INSET;
  let shares_typing = |beat: &Beat| {
    beat.area.left >= frame.left + inset
      && beat.area.right <= frame.right - inset
      && beat.area.top >= frame.top + inset
      && beat.area.bottom <= frame.bottom - inset
  };
  if typed.is_some() && !beats.iter().all(|beat| beat.typed || shares_typing(beat)) {
    return None;
  }
  let first_ms = beats.first()?.start_ms;
  let last_ms = beats.last()?.end_ms;
  // A shot never waits out the pointer working somewhere it does not show.
  if away(signals, frame, first_ms, last_ms)
    .iter()
    .any(|&(from, to)| to - from > MAX_AWAY_MS)
  {
    return None;
  }
  let cap = |ms: u64| {
    if duration_ms > 0 {
      ms.min(duration_ms)
    } else {
      ms
    }
  };
  let start_ideal = first_ms.saturating_sub(LEAD_MS).max(not_before);
  let start_latest = first_ms.saturating_sub(TRANSITION_MS);
  let arrived = start_latest.max(not_before);
  // Once the pointer has left, the shot leaves with it, as soon as its last
  // beat has had its shortest hold, however short that leaves the shot.
  let left_ms = away(signals, frame, last_ms, cap(u64::MAX))
    .into_iter()
    .find(|&(from, to)| to - from >= LEFT_MS)
    .map(|(from, _)| from);
  let leave_by = left_ms.map_or(u64::MAX, |left| {
    cap((left + TRANSITION_MS).max(last_ms + MIN_TAIL_MS))
  });
  let end_earliest = cap((last_ms + MIN_TAIL_MS).max(arrived + MIN_CLIP_MS)).min(leave_by);
  let end_ideal = cap((last_ms + TAIL_MS).max(start_ideal + MIN_CLIP_MS))
    .min(leave_by)
    .max(end_earliest);
  let handover_earliest =
    cap((last_ms + HANDOVER_HOLD_MS).max(arrived + MIN_HANDOVER_CLIP_MS)).min(leave_by);
  // Arriving late, it still holds for as long as the shortest shot.
  let start_late = end_earliest.saturating_sub(MIN_CLIP_MS).max(arrived);
  let weight: f64 = beats.iter().map(|beat| beat.weight).sum();
  let outside = outside_seconds(signals, frame, first_ms, last_ms + MIN_TAIL_MS);
  Some(Candidate {
    start_ideal,
    start_latest,
    start_late,
    end_ideal,
    end_earliest,
    handover_earliest,
    focus,
    zoom,
    score: weight * zoom.log2() - 2.0 * TRANSITION_COST - COVER_COST * outside,
    frame,
    left_ms,
  })
}

/// What starting `candidate` at `start_ms` costs for arriving after its work
/// began.
pub(super) fn lateness(candidate: &Candidate, start_ms: u64) -> f64 {
  LATE_COST * start_ms.saturating_sub(candidate.start_latest) as f64 / 1_000.0
}

/// Whether `next` follows `previous` too closely for the whole screen to
/// show between them, so the one hands over to the other.
pub(super) fn butts(previous: &Candidate, next: &Candidate) -> bool {
  next.start_ideal < previous.end_ideal + MIN_OVERVIEW_MS
}

/// Where `previous` hands over to `next` when they butt.
pub(super) fn handover(previous: &Candidate, next: &Candidate) -> u64 {
  next
    .start_ideal
    .max(previous.handover_earliest)
    .min(next.start_latest)
}

/// What going from `previous` to `next` adds to the score, or `None` where
/// they are too close to pan between and too close to zoom out between, or
/// where holding `previous` until the pan would wait out the pointer working
/// somewhere neither shows.
pub(super) fn link(previous: &Candidate, next: &Candidate, signals: &Signals) -> Option<f64> {
  if !butts(previous, next) {
    let gap = next.start_ideal - previous.end_ideal;
    let brief =
      CALM_OVERVIEW_MS.saturating_sub(gap) as f64 / (CALM_OVERVIEW_MS - MIN_OVERVIEW_MS) as f64;
    return Some(-PUMP_COST * brief.min(1.0));
  }
  if previous.handover_earliest > next.start_latest {
    return None;
  }
  if let Some(left) = previous.left_ms {
    let pan = handover(previous, next);
    let unseen = away(signals, previous.frame, left, pan)
      .into_iter()
      .any(|(from, to)| {
        away(signals, next.frame, from, to)
          .iter()
          .any(|&(from, to)| to - from > MAX_AWAY_MS)
      });
    if unseen {
      return None;
    }
  }
  // In frames: how far the middle travels, and how many doublings of zoom.
  let frame = (1.0 / previous.zoom).max(1.0 / next.zoom);
  let moved = (next.focus.0 - previous.focus.0).hypot(next.focus.1 - previous.focus.1) / frame
    + (previous.zoom / next.zoom).log2().abs();
  let twitch = if moved < MICRO_MOVE { MICRO_COST } else { 0.0 };
  // One handover takes the place of a leaving and an arrival.
  Some(TRANSITION_COST - PAN_COST * moved - twitch)
}
