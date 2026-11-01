// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useSyncExternalStore } from "react";

import { type Playhead } from "../../timeline/scrub-playhead";

import { recordingPinView } from "./recording-annotation-pin-view";
import { isPinnable } from "./recording-annotation-pins";
import { RecordingAnnotationClip } from "./recording-annotations";
import { actOnPin, AnnotationClipPinning } from "./use-annotation-clip-menu";
import { RecordingPinStatus } from "./use-recording-pin-status";

import type { AnnotationPins } from "../../annotations/published-annotation";

/**
 * The recording's pins as the chosen annotation's island shows and changes
 * them. How a pin stands depends on the playhead, which is kept out of React
 * (`scrub-playhead.ts`), so the playhead is heard here and a render asked
 * for only when what the pins show at it changes: as it crosses into or out
 * of a stretch out of view or one the tracker is unsure of, not on every
 * frame it plays. `getPositionMs` is the playhead in source time, which the
 * tracker's stretches are in.
 */
export function useRecordingAnnotationPins({
  clips,
  getPositionMs,
  pinStatus,
  pinning,
  playhead,
}: {
  clips: RecordingAnnotationClip[];
  getPositionMs: () => number;
  pinStatus: ReadonlyMap<string, RecordingPinStatus>;
  pinning: AnnotationClipPinning;
  playhead: Playhead;
}): AnnotationPins {
  const viewOf = (id: string) => {
    const clip = clips.find((item) => item.annotation.id === id);
    if (!clip || !isPinnable(clip)) return undefined;
    if (!clip.pin) return null;
    return recordingPinView({
      canHide: pinning.canHideHere(id),
      canShow: pinning.canShowHere(id),
      pin: clip.pin,
      positionMs: getPositionMs(),
      status: pinStatus.get(id),
    });
  };
  // What the pins show at the playhead, as text: the playhead's every move
  // reads it, and only a change renders.
  useSyncExternalStore(playhead.subscribe, () =>
    JSON.stringify(
      clips.flatMap((clip) =>
        clip.pin ? [viewOf(clip.annotation.id) ?? null] : [],
      ),
    ),
  );
  return {
    apply: (id, action) => {
      actOnPin(pinning, id, action);
    },
    viewOf,
  };
}
