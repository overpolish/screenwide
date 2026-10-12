// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Platform-neutral semantic events emitted by interactive OSC sessions.

use serde::Serialize;

use super::{
  controller::ControllerEvent,
  geometry::{Handle, Rect},
  gesture::GestureKind,
};

pub const REGION_EVENT: &str = "screenshot-region-osc";
pub const DESKTOP_LAYOUT_EVENT: &str = "screenshot-region-desktop-layout";

#[derive(Clone, Copy, Debug, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SemanticStatus {
  Changed,
  Finished,
  Cancelled,
  Layout,
}

#[derive(Clone, Copy, Debug, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SemanticGesture {
  Drawing,
  Moving,
  Resizing { handle: SemanticHandle },
}

#[derive(Clone, Copy, Debug, Serialize, PartialEq)]
#[repr(u8)]
#[serde(rename_all = "lowercase")]
pub enum SemanticHandle {
  Body = 1,
  North = 2,
  South = 3,
  East = 4,
  West = 5,
  NorthEast = 6,
  NorthWest = 7,
  SouthEast = 8,
  SouthWest = 9,
}

#[derive(Clone, Copy, Debug, Serialize, PartialEq)]
pub struct SemanticRegion {
  pub x: f64,
  pub y: f64,
  pub width: f64,
  pub height: f64,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SemanticEvent {
  pub status: SemanticStatus,
  pub gesture: Option<SemanticGesture>,
  pub region: Option<SemanticRegion>,
  pub monitor_id: Option<u32>,
}

fn region(rect: Rect) -> SemanticRegion {
  SemanticRegion {
    x: rect.origin.x,
    y: rect.origin.y,
    width: rect.size.width,
    height: rect.size.height,
  }
}

pub fn semantic_handle(handle: Handle) -> SemanticHandle {
  match handle {
    Handle::Body => SemanticHandle::Body,
    Handle::North => SemanticHandle::North,
    Handle::South => SemanticHandle::South,
    Handle::East => SemanticHandle::East,
    Handle::West => SemanticHandle::West,
    Handle::NorthEast => SemanticHandle::NorthEast,
    Handle::NorthWest => SemanticHandle::NorthWest,
    Handle::SouthEast => SemanticHandle::SouthEast,
    Handle::SouthWest => SemanticHandle::SouthWest,
  }
}

fn semantic_gesture(kind: GestureKind) -> SemanticGesture {
  match kind {
    GestureKind::Drawing => SemanticGesture::Drawing,
    GestureKind::Moving => SemanticGesture::Moving,
    GestureKind::Resizing(handle) => SemanticGesture::Resizing {
      handle: semantic_handle(handle),
    },
  }
}

pub fn event_payload(event: &ControllerEvent, monitor_id: Option<u32>) -> SemanticEvent {
  match event {
    ControllerEvent::Changed { draft, kind } => SemanticEvent {
      status: SemanticStatus::Changed,
      gesture: Some(semantic_gesture(*kind)),
      region: draft.map(region),
      monitor_id,
    },
    ControllerEvent::Finished { committed, kind } => SemanticEvent {
      status: SemanticStatus::Finished,
      gesture: Some(semantic_gesture(*kind)),
      region: committed.map(region),
      monitor_id,
    },
    ControllerEvent::Cancelled { committed } => SemanticEvent {
      status: SemanticStatus::Cancelled,
      gesture: None,
      region: committed.map(region),
      monitor_id,
    },
  }
}
