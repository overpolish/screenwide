// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Who holds the island. It is one window whatever it shows, and more than
//! one feature can want it at once: notes run while a screenshot is copied,
//! a dictation comes up over a recording. Each holds a claim for as long as
//! it has something on the island, and the window follows the claims alone:
//! out while any is held, hanging where the one on top hangs, and tucked
//! away once the last is let go. No feature shows or hides the window
//! itself, so letting go of its own claim never takes the island from
//! another.
//!
//! The island can also stand aside for a moment with its claims kept, for a
//! capture that must not find it on screen or a region dragged across the
//! top of the screen. It comes back when asked, or when something new
//! claims it.
//!
//! The page is sent the claims, top first (`island://claims`, and
//! `island_claims` for a page that loads late), and shows the island of the
//! one on top, so this order is the only one there is.
//!
//! A new island: add its claim below in the order it stands over the
//! others, then map it to what the page draws (`island-kinds.ts`, which
//! does not compile until every claim is mapped). A part whose state is one
//! value holds the island with an `IslandSlot`, which sends the value and
//! claims and lets go with it; anything else calls `claim` and `release`
//! itself.

use std::sync::{Mutex, MutexGuard, PoisonError};

use serde::Serialize;
use tauri::{AppHandle, Emitter};
use ts_rs::TS;

use super::super::island_placement::IslandPlacement;
use crate::app_windows::WindowLabel;

const EVENT: &str = "island://claims";

/// What can hold the island, the first on top. The claim on top is what the
/// island shows and sets where it hangs.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum IslandClaim {
  ScrollingCapture,
  Dictation,
  ReferenceDrop,
  QuickScreenshot,
  CopiedScreenshot,
  TextRecognition,
  Ruler,
  /// Notes, which show on the recording's island and go on under the other
  /// tools.
  Notes,
  /// A recording, from its start until the Editor takes it or the controls
  /// come back.
  Recording,
  Annotate,
  /// A new version offered, which only an island free of everything else
  /// shows: any other claim ends the offer.
  Update,
  /// The recording controls.
  Controls,
}

/// What the window does once the claims change.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Move {
  /// Out, hanging there: something new is shown.
  Out(IslandPlacement),
  /// Back to what is left, hanging there: what was on top went.
  HandBack(IslandPlacement),
  /// Tucked away: nothing holds the island.
  Away,
  /// Out of sight for a moment, its claims kept.
  Aside { at_once: bool },
  /// As it is.
  Stay,
}

#[derive(Debug)]
struct Claims {
  /// What holds the island, top first, each with where it hangs it.
  held: Vec<(IslandClaim, IslandPlacement)>,
  /// Standing aside: out of sight, though held.
  aside: bool,
}

impl Claims {
  const fn new() -> Self {
    Self {
      held: Vec::new(),
      aside: false,
    }
  }

  fn top(&self) -> Option<IslandPlacement> {
    self.held.first().map(|&(_, placement)| placement)
  }

  fn holds(&self, claim: IslandClaim) -> bool {
    self.held.iter().any(|&(held, _)| held == claim)
  }

  fn list(&self) -> Vec<IslandClaim> {
    self.held.iter().map(|&(claim, _)| claim).collect()
  }

  fn claim(&mut self, claim: IslandClaim, placement: IslandPlacement) -> Move {
    if claim != IslandClaim::Update {
      self.held.retain(|&(held, _)| held != IslandClaim::Update);
    }
    if let Some((_, held)) = self.held.iter_mut().find(|(held, _)| *held == claim) {
      // Held already: only where it hangs can change, which moves the island
      // if it is out and this claim is on top.
      if *held == placement {
        return Move::Stay;
      }
      *held = placement;
      if self.aside {
        return Move::Stay;
      }
    } else {
      let at = self.held.partition_point(|&(held, _)| held < claim);
      self.held.insert(at, (claim, placement));
      self.aside = false;
    }
    self.top().map_or(Move::Stay, Move::Out)
  }

  fn release(&mut self, claim: IslandClaim) -> Move {
    let Some(at) = self.held.iter().position(|&(held, _)| held == claim) else {
      return Move::Stay;
    };
    self.held.remove(at);
    match self.top() {
      None => {
        self.aside = false;
        Move::Away
      }
      Some(_) if self.aside => Move::Stay,
      Some(placement) => Move::HandBack(placement),
    }
  }

  fn stand_aside(&mut self, at_once: bool) -> Move {
    self.aside = !self.held.is_empty();
    Move::Aside { at_once }
  }

  fn come_back(&mut self) -> Move {
    if !self.aside {
      return Move::Stay;
    }
    self.aside = false;
    self.top().map_or(Move::Away, Move::Out)
  }
}

static CLAIMS: Mutex<Claims> = Mutex::new(Claims::new());

fn claims() -> MutexGuard<'static, Claims> {
  CLAIMS.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Whether `claim` holds the island.
pub(crate) fn holds(claim: IslandClaim) -> bool {
  claims().holds(claim)
}

/// Whether nothing holds the island.
pub(crate) fn is_unclaimed() -> bool {
  claims().held.is_empty()
}

/// `claim` takes the island, at the top of the screen.
pub(crate) fn claim(app: &AppHandle, claim: IslandClaim) {
  claim_at(app, claim, IslandPlacement::Top);
}

/// `claim` takes the island, hanging at `placement` while it is on top. A
/// claim held already only moves where it hangs.
pub(crate) fn claim_at(app: &AppHandle, claim: IslandClaim, placement: IslandPlacement) {
  change(app, claim, |claims| claims.claim(claim, placement));
}

/// `claim` lets the island go: it goes back to what still holds it, or
/// tucks away when nothing does.
pub(crate) fn release(app: &AppHandle, claim: IslandClaim) {
  change(app, claim, |claims| claims.release(claim));
}

/// Changes the claims, tells the page if what holds the island changed, and
/// then moves the window: the page knows what to show before it comes out.
fn change(app: &AppHandle, claim: IslandClaim, how: impl FnOnce(&mut Claims) -> Move) {
  let (change, held) = {
    let mut claims = claims();
    let before = claims.list();
    let change = how(&mut claims);
    let after = claims.list();
    (change, (after != before).then_some(after))
  };
  if let Some(held) = held {
    let _ = app.emit_to(WindowLabel::Island.as_str(), EVENT, held);
  }
  if let Err(error) = run(app, change) {
    eprintln!("Could not move the island for {claim:?}: {error}");
  }
}

/// What holds the island, top first, for a page that loads after it was
/// told (`island://claims`).
#[tauri::command]
pub fn island_claims() -> Vec<IslandClaim> {
  claims().list()
}

/// The island goes out of sight with its claims kept, `at_once` for a
/// capture that follows straight after, until `come_back` or a new claim.
pub(crate) fn stand_aside(app: &AppHandle, at_once: bool) {
  let change = claims().stand_aside(at_once);
  if let Err(error) = run(app, change) {
    eprintln!("Could not put the island aside: {error}");
  }
}

/// The island comes back from standing aside, for whatever still holds it.
pub(crate) fn come_back(app: &AppHandle) {
  let change = claims().come_back();
  if let Err(error) = run(app, change) {
    eprintln!("Could not bring the island back: {error}");
  }
}

// Run with the claims unlocked: showing and hiding wait on the main thread,
// which may itself be waiting to change a claim.
fn run(app: &AppHandle, change: Move) -> tauri::Result<()> {
  match change {
    Move::Out(placement) => super::place(app, placement).and_then(|()| super::show(app)),
    // Tucking away, or handing over already, it comes out again where it
    // now belongs rather than finishing going.
    Move::HandBack(placement) if super::is_shown() => super::hand_over(app, placement),
    Move::HandBack(placement) => super::place(app, placement).and_then(|()| super::show(app)),
    Move::Away | Move::Aside { at_once: false } => super::hide(app),
    Move::Aside { at_once: true } => super::hide_at_once(app),
    Move::Stay => Ok(()),
  }
}

#[cfg(test)]
mod tests;
