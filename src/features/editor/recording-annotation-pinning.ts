// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";

import { AnnotationClipPinning } from "./components/use-annotation-clip-menu";
import {
  clearedPinCorrections,
  isPinnable,
  onPinnedFrame,
  outOfViewStretches,
  pinnedClip,
  unpinnedClip,
  withPinKeyframe,
} from "./recording-annotation-pins";
import { RecordingAnnotationClip } from "./recording-annotations";
import { RecordingPinStatus } from "./use-recording-pin-status";

/**
 * What a recording annotation's menu does to its pin, against the clips as
 * they stand. `withClip` writes one clip's change back as an edit; the
 * playhead, in source time, is where a pin, a correction or an out-of-view
 * stretch is made.
 */
export function recordingAnnotationPinning({
  clips,
  getPositionMs,
  pinStatus,
  sessionId,
  withClip,
}: {
  clips: RecordingAnnotationClip[];
  getPositionMs: () => number;
  pinStatus: ReadonlyMap<string, RecordingPinStatus>;
  sessionId: number | null;
  withClip: (
    id: string,
    change: (clip: RecordingAnnotationClip) => RecordingAnnotationClip,
  ) => void;
}): AnnotationClipPinning {
  // Hidden by tracking where its content went out of sight, or by the hand.
  const hiddenAt = (id: string, position: number) => {
    const clip = clips.find((item) => item.annotation.id === id);
    const stretches = [
      ...(pinStatus.get(id)?.covered ?? []),
      ...(clip?.pin ? outOfViewStretches(clip.pin, clip.endMs) : []),
    ];
    return stretches.some(
      ([start, end]) => position >= start && position < end,
    );
  };
  return {
    // Shown, inside its clip, and off the frame it was pinned on, which stays
    // a keyframe it is seen on.
    canHideHere: (id) => {
      const clip = clips.find((item) => item.annotation.id === id);
      const position = getPositionMs();
      return (
        clip?.pin !== undefined &&
        position >= clip.startMs &&
        position < clip.endMs &&
        !onPinnedFrame(clip.pin, position) &&
        !hiddenAt(id, position)
      );
    },
    canShowHere: (id) => hiddenAt(id, getPositionMs()),
    onClearCorrections: (id) => {
      withClip(id, (clip) =>
        clip.pin ? { ...clip, pin: clearedPinCorrections(clip.pin) } : clip,
      );
    },
    onHideHere: (id) => {
      const ms = Math.round(getPositionMs());
      withClip(id, (clip) =>
        clip.pin
          ? {
              ...clip,
              pin: withPinKeyframe(clip.pin, {
                dx: 0,
                dy: 0,
                ms,
                outOfView: true,
              }),
            }
          : clip,
      );
    },
    onPinnedChange: (id, pinned) => {
      withClip(id, (clip) =>
        pinned
          ? clip.pin || !isPinnable(clip)
            ? clip
            : pinnedClip(clip, getPositionMs())
          : unpinnedClip(clip),
      );
    },
    // The native side finds the content on this frame, near where the
    // annotation was last shown, or keeps it there when unsure; the size
    // holds, as only a resize changes it.
    onShowHere: (id) => {
      if (sessionId === null) return;
      const ms = Math.round(getPositionMs());
      void invoke<[number, number] | null>(
        "recording_preview_pin_back_in_view",
        {
          annotationId: id,
          sessionId,
          sourceMs: ms,
        },
      )
        .then((seen) => {
          if (!seen) return;
          withClip(id, (clip) =>
            clip.pin
              ? {
                  ...clip,
                  pin: withPinKeyframe(clip.pin, {
                    dx: seen[0],
                    dy: seen[1],
                    ms,
                  }),
                }
              : clip,
          );
        })
        .catch((cause: unknown) => {
          console.error("Could not show the annotation here", cause);
        });
    },
  };
}
