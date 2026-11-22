// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import {
  createRecordingTimelineEdit,
  type RecordingTimelineEdit,
} from "../editing/recording-timeline-edit";

import { audioClipWriter } from "./audio-clip-writer";

import type { RecordingAudioClip } from "../../../../bindings/RecordingAudioClip";

const song: RecordingAudioClip = {
  endMs: 27_945.999_999_999_993,
  id: "song",
  isInstrumental: false,
  isLooped: false,
  kind: "music",
  lane: 0,
  offsetMs: 0.4,
  sound: "song.mp3",
  startMs: 1_204.6,
  volumeDb: 0,
};

const written = (
  write: (writer: ReturnType<typeof audioClipWriter>) => void,
) => {
  let last: RecordingTimelineEdit | null = null;
  const keep = (next: RecordingTimelineEdit) => {
    last = next;
  };
  const writer = audioClipWriter({
    current: {
      edit: { ...createRecordingTimelineEdit(1), audioClips: [song] },
      steps: {
        once: keep,
        step: (next) => {
          if (typeof next !== "function") keep(next);
        },
      },
    },
  });
  write(writer);
  return (last as RecordingTimelineEdit | null)?.audioClips?.[0];
};

describe("audioClipWriter", () => {
  // A song dropped on a timeline with squeezed lulls ends at a fraction of a
  // millisecond, which Rust refuses, so the edit was never saved or played.
  it("keeps clips in whole milliseconds", () => {
    expect(
      written((writer) => {
        writer.write([song]);
      }),
    ).toMatchObject({
      endMs: 27_946,
      offsetMs: 0,
      startMs: 1_205,
    });
    expect(
      written((writer) => {
        writer.update("song", (clip) => ({ ...clip, endMs: 9_000.5 }), {
          isDragged: true,
        });
      }),
    ).toMatchObject({ endMs: 9_001, startMs: 1_205 });
  });
});
