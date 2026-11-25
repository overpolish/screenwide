// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { formatDifferences } from "./format-differences";
import { resolvedTimelineEdit } from "./format-resolve";
import { withEditsFormatAdded } from "./format-scope";
import { formatSync, withCustomToggled, withShownToggled } from "./format-sync";
import { writtenTimelineEdit } from "./format-write";

import type { FormatContext } from "./format-scope";
import type { FormatEdits } from "../../../../bindings/FormatEdits";
import type { RecordingAudioClip } from "../../../../bindings/RecordingAudioClip";
import type { RecordingTimelineEdit } from "../../timeline/editing/recording-timeline-edit";

const wide = { height: 1_080, width: 1_920 };
const vertical = { height: 1_920, width: 1_080 };

const song: RecordingAudioClip = {
  endMs: 9_000,
  id: "song",
  isInstrumental: false,
  isLooped: false,
  kind: "music",
  lane: 0,
  offsetMs: 0,
  sound: "theme",
  startMs: 0,
  volumeDb: 0,
};

const edit = {
  artifactId: 1,
  audioClips: [song],
  sceneClips: [{ endMs: 4_000, id: "scene", preset: "stacked", startMs: 0 }],
} as unknown as RecordingTimelineEdit;

const shapes = [
  { annotationKey: "wide", formats: ["youtube"], id: "wide" },
  { annotationKey: "vertical", formats: ["tiktok", "shorts"], id: "vertical" },
];
const inFormat = (id: string): FormatContext => {
  const shape = id === "youtube" ? "wide" : "vertical";
  return { annotationKey: shape, id, shape, shapes };
};
const resolve = (
  edits: FormatEdits,
  id: string,
  stored: RecordingTimelineEdit = edit,
) =>
  resolvedTimelineEdit(stored, edits, {
    base: wide,
    canvas: id === "youtube" ? wide : vertical,
    context: inFormat(id),
  });

describe("songs and sound effects per format", () => {
  it("deletes a song from every format, whichever it is deleted in", () => {
    const previous = resolve({}, "tiktok");
    const written = writtenTimelineEdit({
      context: inFormat("tiktok"),
      edits: {},
      next: { ...previous, audioClips: [] },
      previous,
      scope: "shape",
      stored: edit,
    });
    expect(written.edit.audioClips).toEqual([]);
  });

  it("leaves a song out of one platform, playing in its shape's others", () => {
    const edits = withShownToggled("tiktok", {
      context: inFormat("shorts"),
      edit,
      edits: {},
      id: "song",
    });
    expect(resolve(edits, "tiktok").audioClips).toHaveLength(0);
    expect(resolve(edits, "shorts").audioClips).toHaveLength(1);
  });

  it("plays a song added in any shape in every format", () => {
    const previous = resolve({}, "tiktok");
    const written = writtenTimelineEdit({
      context: inFormat("tiktok"),
      edits: {},
      next: {
        ...previous,
        audioClips: [...(previous.audioClips ?? []), { ...song, id: "own" }],
      },
      previous,
      scope: "shape",
      stored: edit,
    });
    expect(
      resolve(written.edits, "youtube", written.edit).audioClips?.map(
        (clip) => clip.id,
      ),
    ).toEqual(["song", "own"]);
  });
});

describe("bringing formats in step", () => {
  const onlyYoutube: FormatEdits = {
    audio: { song: { showIn: { formats: ["youtube"] } } },
  };

  it("draws what a format leaves out as a ghost, with where it shows", () => {
    const hidden = formatDifferences(
      edit,
      onlyYoutube,
      inFormat("tiktok"),
    ).hidden;
    expect(hidden.audio.map((clip) => clip.id)).toEqual(["song"]);
    expect(hidden.shownIn.song).toEqual(["youtube"]);
  });

  it("ticks each format a left-out song shows in", () => {
    expect(
      formatSync({
        context: inFormat("tiktok"),
        edit,
        edits: onlyYoutube,
        id: "song",
      })?.formats,
    ).toEqual([
      { id: "youtube", isShown: true },
      { id: "tiktok", isShown: false },
      { id: "shorts", isShown: false },
    ]);
  });

  it("clears where a song shows once it shows everywhere", () => {
    const once = withShownToggled("tiktok", {
      context: inFormat("tiktok"),
      edit,
      edits: onlyYoutube,
      id: "song",
    });
    const twice = withShownToggled("shorts", {
      context: inFormat("tiktok"),
      edit,
      edits: once,
      id: "song",
    });
    expect(twice.audio).toEqual({});
  });

  it("never leaves something out of the only format it shows in", () => {
    const edits = withShownToggled("youtube", {
      context: inFormat("tiktok"),
      edit,
      edits: onlyYoutube,
      id: "song",
    });
    expect(edits).toBe(onlyYoutube);
  });

  it("keeps a custom scene's changes in its shape, even the main one", () => {
    const shown = resolve({}, "youtube");
    const custom = withCustomToggled({
      context: inFormat("youtube"),
      edits: {},
      id: "scene",
      shown,
    });
    const written = writtenTimelineEdit({
      context: inFormat("youtube"),
      edits: custom,
      next: {
        ...shown,
        sceneClips: shown.sceneClips?.map((clip) => ({
          ...clip,
          preset: "full" as const,
        })),
      },
      previous: shown,
      scope: "all",
      stored: edit,
    });
    const preset = (id: string) =>
      resolve(written.edits, id, written.edit).sceneClips?.[0]?.preset;
    expect(preset("youtube")).toBe("full");
    expect(preset("tiktok")).toBe("stacked");
  });

  it("shows what is left out of one platform in a platform added later", () => {
    const left: FormatEdits = {
      audio: { song: { showIn: { formats: ["youtube", "shorts"] } } },
      captions: { showIn: { formats: ["youtube"] } },
    };
    const added = withEditsFormatAdded({ edits: left }, "reels")?.edits;
    expect(added?.audio?.song.showIn).toEqual({
      formats: ["youtube", "shorts", "reels"],
    });
    expect(added?.captions?.showIn).toEqual({ formats: ["youtube", "reels"] });
  });
});
