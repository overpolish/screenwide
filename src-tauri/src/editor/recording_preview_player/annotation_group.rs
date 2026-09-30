// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Several annotations chosen together, carried on the picture as one.
//!
//! A group's members may be on either pane and need not be showing at the
//! playhead, so the carry works on the clips rather than on what a pane is
//! drawing: every member's clip is written through while the hand moves, and
//! the release commits them all as one edit.

use super::gesture::{provisional_clips, track, Commit};
use super::*;
use crate::editor::annotations::group::{group_bounds, GroupMember, GroupMove};

pub(super) struct GroupDrag {
  pane: u32,
  position: u64,
  before: Vec<RecordingAnnotationClip>,
  moving: GroupMove,
  field: SnapField,
}

impl PreviewPlayerManager {
  /// How wide pane `pane`'s picture is drawn: in output pixels, which every
  /// pane shares on screen, and in points of a style's size. Zero where the
  /// composition is not known yet.
  pub(super) fn pane_image_widths(&self, pane: u32) -> (f64, f64) {
    self
      .selection_composition()
      .map(|composition| {
        let output = if pane == 1 {
          &composition.recording_output.camera
        } else {
          &composition.recording_output.primary
        };
        (output.image_width, output.size_image_width())
      })
      .unwrap_or_default()
  }

  /// The ids chosen together, when two or more are.
  pub(super) fn annotation_group(&self) -> &[String] {
    if self.annotation.selected.len() > 1 {
      &self.annotation.selected
    } else {
      &[]
    }
  }

  /// One sample of the chosen group carried by a press on `pane`. `x` and
  /// `y` are normalised over that pane's source.
  #[allow(clippy::too_many_arguments)]
  pub(super) fn group_gesture(
    &mut self,
    phase: SelectionGesturePhase,
    pane: u32,
    x: f64,
    y: f64,
    snap: u32,
    image_points: f64,
  ) -> Option<Commit> {
    let (clips, duration_ms) = {
      let sources = self.sources.as_ref()?;
      (Arc::clone(&sources.annotation_clips), sources.duration_ms)
    };
    if matches!(phase, SelectionGesturePhase::Cancel) {
      let drag = self.annotation.group.take()?;
      *clips.write().ok()? = drag.before;
      self.settle_group(drag.pane);
      return None;
    }
    if matches!(phase, SelectionGesturePhase::Begin) {
      let position = self.position_ms.min(duration_ms.saturating_sub(1));
      self.annotation.pane = Some(pane);
      self.annotation.group = Some(self.begin_group(pane, position, x, y)?);
      return None;
    }
    let drag = self.annotation.group.as_ref()?;
    let (pane, position) = (drag.pane, drag.position);
    let source = self.pane_source(pane)?;
    let request = threshold_source_px(source.0, image_points).map(|threshold| SnapRequest {
      field: &drag.field,
      threshold,
    });
    let (moved, result) = drag.moving.update(
      source_point(x, y, source),
      SnapModifiers::from_bits(snap),
      request,
    );
    *clips.write().ok()? = provisional_clips(&drag.before, &moved, pane, position);
    if !matches!(phase, SelectionGesturePhase::End) {
      self.publish_annotation_snap(source, &result);
      self.publish_annotation_handles();
      if !self.redraw_annotation_frame(pane, position) {
        let _ = self.restart(PlaybackMode::InteractiveStill);
      }
      return None;
    }
    self.annotation.group = None;
    self.settle_group(pane);
    self.commit(
      pane,
      position,
      moved,
      self.annotation.selected.clone(),
      None,
    )
  }

  pub(super) fn pane_source(&self, pane: u32) -> Option<(u32, u32)> {
    let source = self
      .sources
      .as_ref()?
      .playback_layout
      .panes
      .get(pane as usize)?;
    Some((source.source_width, source.source_height))
  }

  /// Source pixels per output pixel on `pane`, or `None` where either size
  /// is unknown.
  fn pane_density(&self, pane: u32) -> Option<f64> {
    let (image_width, _) = self.pane_image_widths(pane);
    let source = self.pane_source(pane)?;
    (image_width.is_finite() && image_width > 0.0).then(|| f64::from(source.0.max(1)) / image_width)
  }

  fn begin_group(&self, pane: u32, position: u64, x: f64, y: f64) -> Option<GroupDrag> {
    let chosen = self.annotation_group();
    let source = self.pane_source(pane)?;
    let before = self.sources.as_ref()?.annotation_clips.read().ok()?.clone();
    let pressed = self.pane_density(pane);
    // Pinned annotations follow their content on their own and are never
    // chosen with others; one that gained a pin since is left where it is.
    let members: Vec<GroupMember> = before
      .iter()
      .filter(|clip| clip.pin.is_none() && chosen.contains(&clip.annotation.id))
      .map(|clip| {
        let own = if clip.track_id == track(1) { 1 } else { 0 };
        GroupMember {
          before: clip.annotation.clone(),
          scale: pressed
            .zip(self.pane_density(own))
            .map_or(1.0, |(pressed, own)| own / pressed),
        }
      })
      .collect();
    if members.is_empty() {
      return None;
    }
    let (_, size_width) = self.pane_image_widths(pane);
    let per_size = source_per_size(source, size_width);
    let on_pane: Vec<&Annotation> = before
      .iter()
      .filter(|clip| clip.track_id == track(pane))
      .map(|clip| &clip.annotation)
      .filter(|annotation| {
        members
          .iter()
          .any(|member| member.before.id == annotation.id)
      })
      .collect();
    let held = group_bounds(on_pane, per_size);
    let moving = GroupMove::new(source_point(x, y, source), members, held);
    let shown = self.pane_annotations(pane);
    let field = SnapField::without(source, &shown, |id| moving.holds(id), size_width);
    Some(GroupDrag {
      pane,
      position,
      before,
      moving,
      field,
    })
  }

  /// Puts the snap chrome away and brings the chrome and the picture back in
  /// step once a carry is over.
  fn settle_group(&mut self, pane: u32) {
    if let Some(source) = self.pane_source(pane) {
      self.publish_annotation_snap(source, &SnapResult::default());
    }
    self.publish_annotation_handles();
    let _ = self.restart(PlaybackMode::InteractiveStill);
  }
}

impl PreviewPlayerManager {
  /// The boxes the chrome draws for the chosen group's members on `pane`:
  /// round each one `shown` there, and round all of them, shown or not.
  #[cfg(any(target_os = "macos", target_os = "windows"))]
  pub(super) fn pane_group_boxes(
    &self,
    pane: u32,
    shown: &[Annotation],
    source: (u32, u32),
    size_width: f64,
  ) -> Vec<crate::editor::annotations::group::NativeAnnotationGroupBox> {
    let group = self.annotation_group();
    let Some(Ok(clips)) = self
      .sources
      .as_ref()
      .map(|sources| sources.annotation_clips.read())
    else {
      return Vec::new();
    };
    let members: Vec<&Annotation> = clips
      .iter()
      .filter(|clip| clip.track_id == track(pane) && clip.pin.is_none())
      .map(|clip| &clip.annotation)
      .filter(|annotation| group.contains(&annotation.id))
      .collect();
    crate::editor::annotations::group::group_boxes(
      &members,
      |id| shown.iter().any(|annotation| annotation.id == id),
      source,
      source_per_size(source, size_width),
      pane as i32,
    )
  }
}
