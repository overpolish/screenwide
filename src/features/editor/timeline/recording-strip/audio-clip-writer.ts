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

export type AudioClipWriter = {
  /** Changes the clip `id` as `change` makes it, or takes it away. */
  update: (
    id: string,
    change: (
      clip: RecordingAudioClip,
      others: RecordingAudioClip[],
    ) => RecordingAudioClip | null,
  ) => void;
  /** Puts `next` in place of every audio clip, as one step of history. */
  write: (next: RecordingAudioClip[]) => void;
};

export function audioClipWriter(latestRef: LatestEdit): AudioClipWriter {
  const write = (next: RecordingAudioClip[]) => {
    const { edit, steps } = latestRef.current;
    steps.once({ ...edit, audioClips: next });
  };
  return {
    update: (id, change) => {
      const current = latestRef.current.edit.audioClips ?? [];
      write(
        current.flatMap((clip) =>
          clip.id === id ? (change(clip, current) ?? []) : [clip],
        ),
      );
    },
    write,
  };
}
