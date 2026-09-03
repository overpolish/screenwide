// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Platform-neutral scene records produced by tool workflows and consumed by
//! native compositor adapters. These records describe what should be shown;
//! they deliberately contain no AppKit, Win32, Metal, or DirectX objects.

use super::{
  geometry::Rect,
  style::{overlay_palette, OverlayPalette},
};

use std::ops::{Deref, DerefMut};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RegionChrome {
  pub frame_visible: bool,
  pub handles_visible: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RegionInteraction {
  pub input_enabled: bool,
  pub allow_drawing: bool,
  pub aspect: Option<f64>,
  pub exclusion_rect: Option<Rect>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SnapshotPresentation {
  pub presented: bool,
  pub composited: bool,
}

/// Complete portable presentation state for the Region compositor.
///
/// `region` remains meaningful while hidden so lifecycle transitions can hide
/// and restore the OSC without losing its cutout. `desktop_presented` controls
/// the per-display compositor surfaces independently of the region itself.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RegionScene {
  pub region: Rect,
  pub visible: bool,
  pub chrome: RegionChrome,
  pub interaction: RegionInteraction,
  pub snapshot: SnapshotPresentation,
  pub desktop_presented: bool,
  pub overlay: OverlayPalette,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RegionSceneOwner {
  #[default]
  Normal,
  DormantNormal,
  Screenshot,
  RestoringNormal,
}

/// Keeps the workflow's requested Region scene separate from the scene that
/// is currently projected onto native compositor surfaces.
///
/// Closing the recording controls temporarily projects a normal scene as
/// hidden. Retaining the unprojected request lets opening the controls restore
/// it synchronously without relying on another frontend state change.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RegionSceneState {
  presented: RegionScene,
  requested_normal: RegionScene,
}

impl Default for RegionSceneState {
  fn default() -> Self {
    let presented = RegionScene::default();
    // `RegionScene::default` is drawing-capable because Quick Screenshot is
    // the first workflow that commonly attaches the shared compositor. The
    // separately retained Recording Bar scene must nevertheless be a normal
    // (non-drawing) scene even before that UI has submitted its first update.
    let mut requested_normal = presented;
    requested_normal.interaction.allow_drawing = false;
    Self {
      presented,
      requested_normal,
    }
  }
}

impl Deref for RegionSceneState {
  type Target = RegionScene;

  fn deref(&self) -> &Self::Target {
    &self.presented
  }
}

impl DerefMut for RegionSceneState {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.presented
  }
}

impl RegionSceneState {
  pub const fn presented(self) -> RegionScene {
    self.presented
  }

  pub const fn request_base(self, owner: RegionSceneOwner) -> RegionScene {
    match owner {
      RegionSceneOwner::Screenshot => self.presented,
      RegionSceneOwner::Normal
      | RegionSceneOwner::DormantNormal
      | RegionSceneOwner::RestoringNormal => self.requested_normal,
    }
  }

  pub fn reconcile_request(
    &mut self,
    requested: RegionScene,
    owner: RegionSceneOwner,
  ) -> Option<RegionScene> {
    if !owner.accepts_drawing(requested.interaction.allow_drawing) {
      return None;
    }
    if owner != RegionSceneOwner::Screenshot {
      self.requested_normal = requested;
    }
    requested.reconcile_owner(owner)
  }

  pub fn normal_presentation(self) -> Option<RegionScene> {
    self
      .requested_normal
      .reconcile_owner(RegionSceneOwner::Normal)
  }

  pub fn set_presented(&mut self, scene: RegionScene) {
    self.presented = scene;
  }
}

impl RegionSceneOwner {
  pub const fn accepts_drawing(self, allow_drawing: bool) -> bool {
    match self {
      Self::Screenshot => allow_drawing,
      Self::Normal | Self::DormantNormal | Self::RestoringNormal => !allow_drawing,
    }
  }
}

impl Default for RegionScene {
  fn default() -> Self {
    Self {
      region: Rect::default(),
      visible: false,
      chrome: RegionChrome {
        frame_visible: true,
        handles_visible: true,
      },
      interaction: RegionInteraction {
        input_enabled: false,
        allow_drawing: true,
        aspect: None,
        exclusion_rect: None,
      },
      snapshot: SnapshotPresentation::default(),
      desktop_presented: false,
      overlay: overlay_palette(),
    }
  }
}

impl RegionScene {
  /// Accepts only updates belonging to the current owner. Ignoring stale
  /// updates is essential during restoration: hiding them would erase the
  /// retained screenshot surface before the normal Region scene replaces it.
  pub fn reconcile_owner(mut self, owner: RegionSceneOwner) -> Option<Self> {
    owner
      .accepts_drawing(self.interaction.allow_drawing)
      .then(|| {
        if owner == RegionSceneOwner::DormantNormal {
          self.visible = false;
          self.interaction.input_enabled = false;
          self.desktop_presented = false;
        } else {
          self.desktop_presented = self.visible;
        }
        self
      })
  }
}

#[cfg(test)]
mod tests;
