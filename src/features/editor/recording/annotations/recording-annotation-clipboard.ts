// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  AnnotationChoice,
  AnnotationClipboard,
  copyAnnotations,
  movedAnnotations,
  pastedAnnotations,
} from "../../annotations/annotation-clipboard";
import { AnnotationFrame } from "../../annotations/annotation-pace";
import { Annotation } from "../../annotations/annotations";
import { CaptureSource } from "../../layer-measure";
import { RecordingTimelineEdit } from "../../timeline/editing/recording-timeline-edit";
import {
  recordingTimelineLengthMs,
  recordingTimelineMsAt,
} from "../../timeline/editing/recording-timeline-time";
import { RecordingVideoTrackId } from "../../types";

import {
  RecordingAnnotationClip,
  recordingAnnotationClipAt,
} from "./recording-annotations";

/** How far an arrow key nudges the chosen annotations, in points, and how
 * far with Shift. */
const NUDGE_POINTS = { coarse: 10, fine: 1 } as const;

/** A copy of a clip that keeps its time and picture: free of the pin and
 * scene places that held the original to its content. */
const freeCopy = (
  clip: RecordingAnnotationClip,
  annotation: Annotation,
): RecordingAnnotationClip => {
  const { pin: _pin, sceneShapes: _shapes, ...rest } = clip;
  return { ...rest, annotation };
};

/**
 * Copy, cut, paste, duplicate, choose all and nudge for a recording's
 * annotations. A paste lands at the playhead on the picture in hand, the
 * screen where none is, each clip as long as the one copied; a duplicate
 * keeps its original's time. What a paste or duplicate adds is chosen.
 */
export function recordingAnnotationClipboard({
  choice,
  clips,
  commitAnnotations,
  commitClips,
  edit,
  frames,
  getPositionMs,
  onSelectTrack,
  screenCaptureScale,
  shown,
  sourceDurationMs,
  trackId,
}: {
  choice: AnnotationChoice;
  clips: RecordingAnnotationClip[];
  /** The shown annotations, edited, written back as the lane writes them:
   * over a scene, as that scene's. */
  commitAnnotations: (annotations: Annotation[]) => void;
  commitClips: (clips: RecordingAnnotationClip[]) => void;
  edit: RecordingTimelineEdit;
  frames: Partial<Record<RecordingVideoTrackId, AnnotationFrame>>;
  getPositionMs: () => number;
  screenCaptureScale: number;
  /** The shown annotations, as the picture has them at the playhead. */
  shown: Annotation[];
  sourceDurationMs: number;
  trackId: RecordingVideoTrackId | null;
  onSelectTrack?: (track: RecordingVideoTrackId) => void;
}) {
  const captureOf = (track: RecordingVideoTrackId): CaptureSource | null => {
    const frame = frames[track];
    return frame
      ? {
          height: frame.height,
          scaleFactor: track === "primary" ? screenCaptureScale : 1,
          width: frame.width,
        }
      : null;
  };
  const shownIds = new Set(shown.map(({ id }) => id));
  const chosen = clips.filter(
    (clip) =>
      choice.selectedIds.has(clip.annotation.id) &&
      shownIds.has(clip.annotation.id),
  );
  // Several chosen across both pictures are copied from the picture the
  // first is on: one clipboard holds one capture's points.
  const chosenTrack = chosen.length > 0 ? chosen[0].trackId : null;
  const copiedClips = chosen.filter((clip) => clip.trackId === chosenTrack);
  const chosenCapture = chosenTrack ? captureOf(chosenTrack) : null;
  const clipboardOfChosen = (): AnnotationClipboard | null =>
    chosenCapture && copiedClips.length > 0
      ? {
          capture: chosenCapture,
          items: copiedClips.map((clip) => ({
            annotation: clip.annotation,
            lengthMs: clip.endMs - clip.startMs,
          })),
        }
      : null;
  const copy = () => {
    const clipboard = clipboardOfChosen();
    if (clipboard)
      copyAnnotations(clipboard).catch((cause: unknown) => {
        console.error("Could not copy the annotations", cause);
      });
    return clipboard;
  };
  const onTrack = (track: RecordingVideoTrackId) =>
    clips
      .filter((clip) => clip.trackId === track)
      .map((clip) => clip.annotation);
  const choose = (added: RecordingAnnotationClip[]) => {
    commitClips([...clips, ...added]);
    choice.onSelectedIdsChange(added.map((clip) => clip.annotation.id));
    const track = added.length > 0 ? added[0].trackId : undefined;
    if (track) onSelectTrack?.(track);
  };
  const hasChosen = chosen.length > 0;
  return {
    copy: hasChosen
      ? () => {
          copy();
        }
      : undefined,
    cut: hasChosen
      ? () => {
          if (copy()) choice.deleteSelected();
        }
      : undefined,
    duplicate: hasChosen
      ? () => {
          const clipboard = clipboardOfChosen();
          if (!clipboard || !chosenCapture || !chosenTrack) return;
          const pasted = pastedAnnotations(
            clipboard,
            chosenCapture,
            onTrack(chosenTrack),
          );
          choose(
            copiedClips.map((clip, index) => freeCopy(clip, pasted[index])),
          );
        }
      : undefined,
    hasChosen,
    nudge: (directionX: number, directionY: number, coarse: boolean) => {
      if (!chosenCapture) return;
      const pixels =
        (coarse ? NUDGE_POINTS.coarse : NUDGE_POINTS.fine) *
        chosenCapture.scaleFactor;
      const moved = new Map(
        movedAnnotations(
          shown.filter((annotation) => choice.selectedIds.has(annotation.id)),
          { x: directionX * pixels, y: directionY * pixels },
        ).map((annotation) => [annotation.id, annotation]),
      );
      commitAnnotations(
        shown.map((annotation) => moved.get(annotation.id) ?? annotation),
      );
    },
    paste: (clipboard: AnnotationClipboard) => {
      const track = trackId ?? "primary";
      const capture = captureOf(track);
      if (!capture) return;
      const lengthMs = recordingTimelineLengthMs(edit, sourceDurationMs);
      const pasted = pastedAnnotations(clipboard, capture, onTrack(track));
      choose(
        pasted.map((annotation, index) => {
          const placed = recordingAnnotationClipAt({
            annotation,
            edit,
            frame: frames[track],
            sourceDurationMs,
            sourcePositionMs: getPositionMs(),
            trackId: track,
          });
          const copiedLength = clipboard.items[index]?.lengthMs;
          return copiedLength === undefined
            ? placed
            : {
                ...placed,
                endMs: Math.min(
                  lengthMs,
                  Math.round(placed.startMs + copiedLength),
                ),
              };
        }),
      );
    },
    /** ⌘A: every annotation shown at the playhead. One pinned follows its
     * content on its own and is never chosen with others, so pins are left
     * out of a choice of more than one. */
    selectAll: () => {
      const atMs = recordingTimelineMsAt(
        edit,
        sourceDurationMs,
        getPositionMs(),
      );
      const showing = clips.filter(
        (clip) =>
          shownIds.has(clip.annotation.id) &&
          clip.startMs <= atMs &&
          atMs < clip.endMs,
      );
      const free = showing.filter((clip) => !clip.pin);
      const picked = free.length > 0 ? free : showing.slice(0, 1);
      if (picked.length > 0)
        choice.onSelectedIdsChange(picked.map((clip) => clip.annotation.id));
    },
  };
}
