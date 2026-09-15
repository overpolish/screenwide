// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
#[repr(C)]
pub(super) struct NativeTimedAnnotation {
  annotation: crate::editor::annotations::native::NativeAnnotation,
  start_ms: u64,
  end_ms: u64,
}
const _: () = assert!(std::mem::size_of::<NativeTimedAnnotation>() == 80);

pub(super) fn for_request(request: &CursorExportRequest<'_>) -> Vec<NativeTimedAnnotation> {
  request
    .timeline
    .map_or(&[][..], |t| t.annotation_clips())
    .iter()
    .filter(|clip| clip.track_id == request.annotation_track)
    .map(|clip| {
      let mut annotation = clip.annotation.clone();
      annotation.style.width *= f64::from(request.video.resolution_scale_percent)
        / f64::from(request.video.source_scale_percent.max(1));
      NativeTimedAnnotation {
        annotation: crate::editor::annotations::native::native_annotations(std::slice::from_ref(
          &annotation,
        ))
        .items[0],
        start_ms: clip.start_ms,
        end_ms: clip.end_ms,
      }
    })
    .collect()
}
