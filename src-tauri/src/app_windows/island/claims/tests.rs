// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::super::super::island_placement::LocalRect;
use super::IslandClaim::*;
use super::*;

const TOP: IslandPlacement = IslandPlacement::Top;
const BY_BOX: IslandPlacement = IslandPlacement::ByBox {
  display: 1,
  rect: LocalRect {
    x: 10.0,
    y: 20.0,
    width: 300.0,
    height: 200.0,
  },
};

#[test]
fn notes_take_the_island_back_after_a_screenshot() {
  let mut claims = Claims::new();
  assert_eq!(claims.claim(Notes, TOP), Move::Out(TOP));
  assert_eq!(claims.claim(QuickScreenshot, TOP), Move::Out(TOP));
  assert_eq!(claims.claim(CopiedScreenshot, TOP), Move::Out(TOP));
  assert_eq!(claims.release(QuickScreenshot), Move::HandBack(TOP));
  assert_eq!(claims.release(CopiedScreenshot), Move::HandBack(TOP));
  assert!(claims.holds(Notes));
  assert_eq!(claims.release(Notes), Move::Away);
}

#[test]
fn every_tool_hands_back_to_notes() {
  for tool in [
    Ruler,
    TextRecognition,
    Annotate,
    Dictation,
    ReferenceDrop,
    ScrollingCapture,
    Controls,
    Recording,
  ] {
    let mut claims = Claims::new();
    claims.claim(Notes, TOP);
    claims.claim(tool, TOP);
    assert_eq!(claims.release(tool), Move::HandBack(TOP), "{tool:?}");
  }
}

#[test]
fn the_claim_on_top_sets_where_the_island_hangs() {
  let mut claims = Claims::new();
  claims.claim(Notes, TOP);
  claims.claim(TextRecognition, TOP);
  assert_eq!(claims.claim(TextRecognition, BY_BOX), Move::Out(BY_BOX));
  // A claim under it leaves the island by the box.
  assert_eq!(claims.claim(Dictation, TOP), Move::Out(TOP));
  assert_eq!(claims.release(Dictation), Move::HandBack(BY_BOX));
  assert_eq!(claims.release(TextRecognition), Move::HandBack(TOP));
}

#[test]
fn claiming_again_where_it_hangs_changes_nothing() {
  let mut claims = Claims::new();
  claims.claim(Controls, TOP);
  assert_eq!(claims.claim(Controls, TOP), Move::Stay);
  assert_eq!(claims.release(Ruler), Move::Stay);
}

#[test]
fn standing_aside_keeps_the_claims_until_it_comes_back() {
  let mut claims = Claims::new();
  claims.claim(Notes, TOP);
  claims.claim(Ruler, TOP);
  assert_eq!(claims.stand_aside(true), Move::Aside { at_once: true });
  assert_eq!(claims.release(Ruler), Move::Stay);
  assert_eq!(claims.come_back(), Move::Out(TOP));
  assert_eq!(claims.come_back(), Move::Stay);
}

#[test]
fn a_new_claim_brings_the_island_back_from_aside() {
  let mut claims = Claims::new();
  claims.claim(Notes, TOP);
  claims.stand_aside(true);
  assert_eq!(claims.claim(QuickScreenshot, TOP), Move::Out(TOP));
  assert_eq!(claims.come_back(), Move::Stay);
}

#[test]
fn the_last_release_while_aside_puts_the_island_away() {
  let mut claims = Claims::new();
  claims.claim(Controls, TOP);
  claims.stand_aside(false);
  assert_eq!(claims.release(Controls), Move::Away);
  assert_eq!(claims.come_back(), Move::Stay);
}

#[test]
fn any_other_claim_ends_an_update_offer() {
  let mut claims = Claims::new();
  claims.claim(Update, TOP);
  claims.claim(Notes, TOP);
  assert!(!claims.holds(Update));
  assert_eq!(claims.release(Notes), Move::Away);
}
