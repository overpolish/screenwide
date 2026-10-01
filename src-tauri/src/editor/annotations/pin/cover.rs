// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Still covers followed content goes under: a sticky header, a toolbar, a
//! panel. They are read off the frames themselves. While the content moves,
//! a line of the frame on the content moves with it, and a line of a cover
//! it passes under stays as it was; a cover with nothing on it still shows,
//! since what passed under it is gone from it. Each frame votes on the lines
//! around the content, and a cover's edge is where lines that stay give way
//! to lines that move.

use std::ops::Range;

use super::geometry::Rect;
use super::luma::Plane;

/// A cover's edge, taken only once it has held still.
mod edge;
use edge::Edge;

/// How little a line may change either way, in brightness steps on average,
/// before it is too plain to say whether it moved.
const PLAIN: f32 = 2.0;
/// How far under the other reading one must fall to count.
const CLEAR: f32 = 0.5;
/// The least movement along an axis, in tracking pixels, for a frame to vote
/// on the lines across it.
const MIN_MOTION: f32 = 1.0;
/// How much of its votes a line keeps each time its axis votes, so a header
/// that only starts sticking partway, or scrolls away, is followed within a
/// second or so of moving.
const KEEP: f32 = 0.95;
/// How many votes a line needs to be taken as still or as moving. One frame
/// is enough: a header may stick only a frame before content goes under it.
const MIN_VOTES: f32 = 0.5;
/// How far past the content, in tracking pixels, lines are voted on at least:
/// far enough to take in the edge it is heading for.
const REACH: f32 = 32.0;
/// The fewest pixels across a line worth a vote.
const MIN_SPAN: usize = 4;
/// The fewest still lines in a row taken as a cover, in tracking pixels: a
/// line of content now and then reads still on its own.
const MIN_COVER: usize = 6;
/// How far past a cover's last still line moving content may start, in
/// tracking pixels, for the two to be the cover and what goes under it. The
/// lines between are plain, with nothing on them to hide, and are taken as
/// the cover's.
const GAP: usize = 24;
#[derive(Clone, Copy, PartialEq)]
enum Reading {
  Still,
  Moved,
  Unknown,
}

/// The votes on every line across one axis, rows or columns, and the edges
/// of the covers found on either side of the content along it.
struct Lines {
  still: Vec<f32>,
  moved: Vec<f32>,
  columns: bool,
  before: Edge,
  after: Edge,
}

impl Lines {
  fn new(count: usize, columns: bool) -> Self {
    Self {
      still: vec![0.0; count],
      moved: vec![0.0; count],
      columns,
      before: Edge::default(),
      after: Edge::default(),
    }
  }

  /// Finds the covers' edges again around the content at line `centre`,
  /// after a frame on which it moved `shift` along the axis.
  fn settle(&mut self, centre: f32, shift: f32) {
    let centre = (centre.max(0.0) as usize).min(self.still.len().saturating_sub(1));
    self.before.update(self.edge_before(centre), shift);
    self.after.update(self.edge_after(centre), shift);
  }

  /// The proven edges before and after the content, or neither where they
  /// cross, which no cover leaves.
  fn edges(&self) -> (f32, f32) {
    let before = self.before.proven().unwrap_or(f32::NEG_INFINITY);
    let after = self.after.proven().unwrap_or(f32::INFINITY);
    if before < after {
      (before, after)
    } else {
      (f32::NEG_INFINITY, f32::INFINITY)
    }
  }

  fn reading(&self, line: usize) -> Reading {
    if self.still[line] >= MIN_VOTES {
      Reading::Still
    } else if self.moved[line] >= MIN_VOTES {
      Reading::Moved
    } else {
      Reading::Unknown
    }
  }

  /// Votes on each of `lines`, over `across` of it, where the content moved
  /// `shift` along the axis from `prev` to `next`: still where it matches
  /// the same line before, moved where it matches the line it came from.
  fn vote(
    &mut self,
    prev: &Plane,
    next: &Plane,
    lines: Range<usize>,
    across: Range<usize>,
    shift: f32,
  ) {
    for votes in self.still.iter_mut().chain(self.moved.iter_mut()) {
      *votes *= KEEP;
    }
    let columns = self.columns;
    let at = |plane: &Plane, line: usize, position: usize| {
      if columns {
        plane.data[position * plane.width + line]
      } else {
        plane.data[line * plane.width + position]
      }
    };
    let count = self.still.len();
    let span = across.len() as f32;
    for line in lines {
      let from = line as f32 - shift;
      if from < 0.0 || from.floor() as usize + 1 >= count {
        continue;
      }
      let low = from.floor() as usize;
      let share = from - low as f32;
      let (mut stayed, mut carried) = (0.0, 0.0);
      for position in across.clone() {
        let now = at(next, line, position);
        let came = (1.0 - share) * at(prev, low, position) + share * at(prev, low + 1, position);
        stayed += (now - at(prev, line, position)).abs();
        carried += (now - came).abs();
      }
      let (stayed, carried) = (stayed / span, carried / span);
      if stayed.max(carried) < PLAIN {
        continue;
      }
      // A clear reading overrules what the line did before: a header that
      // scrolled with the page and has just stuck is still now.
      if stayed < CLEAR * carried {
        self.still[line] += 1.0;
        self.moved[line] = 0.0;
      } else if carried < CLEAR * stayed {
        self.moved[line] += 1.0;
        self.still[line] = 0.0;
      }
    }
  }

  fn still(&self, line: usize) -> bool {
    self.reading(line) == Reading::Still
  }

  /// The first line clear of a cover lying before `centre`, towards line
  /// zero: where moving content starts past it.
  fn edge_before(&self, centre: usize) -> Option<usize> {
    let last = (0..=centre).rev().find(|&line| self.still(line))?;
    let mut end = last + 1;
    while end < self.still.len() && self.still(end) {
      end += 1;
    }
    let run = end - (0..end).rev().take_while(|&line| self.still(line)).count();
    if end - run < MIN_COVER {
      return None;
    }
    (end..(end + GAP).min(self.still.len())).find(|&line| self.reading(line) == Reading::Moved)
  }

  /// The first line of a cover lying after `centre`: just past where moving
  /// content ends before it.
  fn edge_after(&self, centre: usize) -> Option<usize> {
    let first = (centre..self.still.len()).find(|&line| self.still(line))?;
    let mut start = first;
    while start > 0 && self.still(start - 1) {
      start -= 1;
    }
    let run = (start..self.still.len())
      .take_while(|&line| self.still(line))
      .count();
    if run < MIN_COVER {
      return None;
    }
    (start.saturating_sub(GAP)..start)
      .rev()
      .find(|&line| self.reading(line) == Reading::Moved)
      .map(|line| line + 1)
  }
}

/// What has been learned of the still covers around followed content.
pub(crate) struct Covers {
  rows: Lines,
  columns: Lines,
}

impl Covers {
  pub(crate) fn new(width: usize, height: usize) -> Self {
    Self {
      rows: Lines::new(height, false),
      columns: Lines::new(width, true),
    }
  }

  /// Votes on the lines around `region`, where the content in it moved
  /// `motion` from `prev` to `next`, in tracking pixels.
  pub(crate) fn watch(&mut self, prev: &Plane, next: &Plane, region: &Rect, motion: [f32; 2]) {
    let span = |from: f32, to: f32, count: usize| {
      (from.max(0.0) as usize).min(count)..(to.max(0.0).ceil() as usize).min(count)
    };
    // Each line is read well past the content on either side: a cover's
    // edge runs straight across, where the outline of something moving in
    // front of a still background does not, and reads mixed.
    if motion[1].abs() >= MIN_MOTION {
      let reach = region.height().max(REACH);
      let across = span(
        region.x0 - region.width(),
        region.x1 + region.width(),
        next.width,
      );
      if across.len() >= MIN_SPAN {
        let lines = span(region.y0 - reach, region.y1 + reach, next.height);
        self.rows.vote(prev, next, lines, across, motion[1]);
        self.rows.settle(region.centre()[1], motion[1]);
      }
    }
    if motion[0].abs() >= MIN_MOTION {
      let reach = region.width().max(REACH);
      let across = span(
        region.y0 - region.height(),
        region.y1 + region.height(),
        next.height,
      );
      if across.len() >= MIN_SPAN {
        let lines = span(region.x0 - reach, region.x1 + reach, next.width);
        self.columns.vote(prev, next, lines, across, motion[0]);
        self.columns.settle(region.centre()[0], motion[0]);
      }
    }
  }

  /// The part of the frame no proven cover lies over, in tracking pixels:
  /// infinite on each side without one.
  pub(crate) fn view(&self) -> Rect {
    let (x0, x1) = self.columns.edges();
    let (y0, y1) = self.rows.edges();
    Rect { x0, y0, x1, y1 }
  }
}
