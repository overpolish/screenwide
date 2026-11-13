// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type RecordingAudioClip } from "../../sound-track/audio-clips";
import { type RecordingTimelineEdit } from "../editing/recording-timeline-edit";

import { type RecordingStripSteps } from "./use-recording-strip-edits";

/** The edit and its history as they are now, read when a change is made,
 * which can be after an import or a picker comes back. */
export type LatestEdit = {
  current: { edit: RecordingTimelineEdit; steps: RecordingStripSteps };
};

/** How a change is kept: on its own, one step of history, or as a move of a
 * drag, whose moves are one step together. */
type WriteOptions = { isDragged?: boolean };

export type AudioClipWriter = {
  /** Changes the clip `id` as `change` makes it, or takes it away. */
  update: (
    id: string,
    change: (
      clip: RecordingAudioClip,
      others: RecordingAudioClip[],
    ) => RecordingAudioClip | null,
    options?: WriteOptions,
  ) => void;
  /** Puts `next` in place of every audio clip. */
  write: (next: RecordingAudioClip[], options?: WriteOptions) => void;
};

export function audioClipWriter(latestRef: LatestEdit): AudioClipWriter {
  const write = (
    next: RecordingAudioClip[],
    { isDragged = false }: WriteOptions = {},
  ) => {
    const { edit, steps } = latestRef.current;
    const changed = { ...edit, audioClips: next };
    if (isDragged) steps.step(changed);
    else steps.once(changed);
  };
  return {
    update: (id, change, options) => {
      const current = latestRef.current.edit.audioClips ?? [];
      write(
        current.flatMap((clip) =>
          clip.id === id ? (change(clip, current) ?? []) : [clip],
        ),
        options,
      );
    },
    write,
  };
}
