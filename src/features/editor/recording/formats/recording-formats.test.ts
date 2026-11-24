// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { defaultCameraOverlay } from "../../export/recording-export-settings";
import { defaultRecordingOutput } from "../../screenshot/screenshot-output";

import { formatDifferences } from "./format-differences";
import { resolvedTimelineEdit } from "./format-resolve";
import { writtenTimelineEdit } from "./format-write";
import {
  addedFormat,
  removedFormat,
  switchedFormat,
  withPhone,
  withPreview,
} from "./recording-formats";

import type { FormatDocument } from "./format-document";
import type { FormatContext } from "./format-scope";
import type { FormatEdits } from "../../../../bindings/FormatEdits";
import type { RecordingAnnotationClip } from "../../../../bindings/RecordingAnnotationClip";
import type { RecordingSceneClip } from "../../../../bindings/RecordingSceneClip";
import type { RecordingTimelineEdit } from "../../timeline/editing/recording-timeline-edit";

const wide = { height: 1_080, width: 1_920 };
const vertical = { height: 1_920, width: 1_080 };

const landscape: FormatDocument = {
  cameraOverlay: defaultCameraOverlay(),
  recordingOutput: defaultRecordingOutput({ primary: wide }),
};

const scene: RecordingSceneClip = {
  endMs: 4_000,
  id: "scene",
  preset: "split-two-thirds",
  startMs: 1_000,
};

const magnifier = {
  annotation: { id: "magnifier" },
  endMs: 3_000,
  startMs: 2_000,
  trackId: 0,
} as unknown as RecordingAnnotationClip;

const edit = {
  annotationClips: [magnifier],
  artifactId: 1,
  sceneClips: [scene],
} as unknown as RecordingTimelineEdit;

const shapes = [
  { annotationKey: "wide", formats: ["youtube-1080"], id: "wide" },
  {
    annotationKey: "vertical",
    formats: ["tiktok", "youtube-shorts"],
    id: "vertical",
  },
];
const inFormat = (id: string): FormatContext => {
  const shape = id === "youtube-1080" ? "wide" : "vertical";
  return { annotationKey: shape, id, shape, shapes };
};
const canvasOf = (id: string) => (id === "youtube-1080" ? wide : vertical);
const resolve = (edits: FormatEdits, id: string) =>
  resolvedTimelineEdit(edit, edits, {
    base: wide,
    canvas: canvasOf(id),
    context: inFormat(id),
  });
const addTikTok = () =>
  addedFormat(null, landscape, {
    preset: "tiktok",
    size: vertical,
  });

describe("recording formats", () => {
  it("adds a tall platform as a shape of its own, previewed on a phone", () => {
    const added = addTikTok();
    expect(added.document.recordingOutput.primary).toMatchObject(vertical);
    expect(added.formats.list).toHaveLength(2);
    expect(added.formats.phone?.phone).toBe("iphone-17-pro");
  });

  it("lays out a platform added from another platform as the main is", () => {
    const tiktok = addTikTok();
    const youtube = addedFormat(tiktok.formats, tiktok.document, {
      preset: "youtube-1080",
      size: wide,
    });
    expect(youtube.document.recordingOutput).toEqual(landscape.recordingOutput);
  });

  it("adds each platform in a shape of its own, clear of its app", () => {
    const tiktok = addTikTok();
    const shorts = addedFormat(tiktok.formats, tiktok.document, {
      preset: "youtube-shorts",
      size: vertical,
    });
    expect(shorts.formats.list.map((shape) => shape.platforms)).toEqual([
      undefined,
      ["tiktok"],
      ["youtube-shorts"],
    ]);
    expect(shorts.document.recordingOutput.primary.clearArea).toEqual({
      bottom: 0.19,
      side: 0.05,
      top: 0.09,
    });
  });

  it("keeps the phone and the preview whichever shape is in hand", () => {
    const added = addTikTok();
    const shown = withPreview(
      withPhone(added.formats, { phone: "pixel-10" }),
      true,
    );
    const back = shown
      ? switchedFormat(shown, added.document, added.formats.list[0]?.id ?? "")
      : null;
    expect(back?.formats.phone?.phone).toBe("pixel-10");
    expect(back?.formats.preview).toBe(true);
  });

  it("brings each shape's own canvas back on a switch", () => {
    const added = addTikTok();
    const back = switchedFormat(
      added.formats,
      added.document,
      added.formats.list[0]?.id ?? "",
    );
    expect(back?.document.recordingOutput.primary).toMatchObject(wide);
    const again = back
      ? switchedFormat(back.formats, back.document, "tiktok")
      : null;
    expect(again?.document.recordingOutput.primary).toMatchObject(vertical);
  });

  it("removes a platform with its shape, and the main canvas stays", () => {
    const tiktok = addTikTok();
    const shorts = addedFormat(tiktok.formats, tiktok.document, {
      preset: "youtube-shorts",
      size: vertical,
    });
    const once = removedFormat(shorts.formats, shorts.document, "tiktok");
    expect(once.formats?.list[1]?.platforms).toEqual(["youtube-shorts"]);
    const twice = once.formats
      ? removedFormat(once.formats, once.document, "youtube-shorts")
      : null;
    expect(twice?.formats?.list).toHaveLength(1);
    expect(twice?.document.recordingOutput.primary).toMatchObject(wide);
  });

  it("adds a platform of the main canvas's proportions as a shape apart", () => {
    const big = { height: 1_800, width: 3_200 };
    const document: FormatDocument = {
      ...landscape,
      recordingOutput: defaultRecordingOutput({ primary: big }),
    };
    const added = addedFormat(null, document, {
      preset: "youtube-1080",
      size: wide,
    });
    expect(added.formats.list.map((shape) => shape.platforms)).toEqual([
      undefined,
      ["youtube-1080"],
    ]);
    expect(added.formats.list[0]?.look?.recordingOutput.primary).toMatchObject(
      big,
    );
    expect(added.document.recordingOutput.primary).toMatchObject(wide);
  });

  it("never takes the main canvas away", () => {
    const tiktok = addTikTok();
    const mainId = tiktok.formats.list[0]?.id ?? "";
    expect(removedFormat(tiktok.formats, tiktok.document, mainId).formats).toBe(
      tiktok.formats,
    );
  });

  it("marks an annotation where it shows alone, not where it is left out", () => {
    const edits: FormatEdits = {
      annotations: { magnifier: { showIn: { formats: ["youtube-1080"] } } },
    };
    expect(
      formatDifferences(edit, edits, inFormat("youtube-1080")).ids,
    ).toEqual(["magnifier"]);
    expect(formatDifferences(edit, edits, inFormat("tiktok")).ids).toEqual([]);
  });

  it("lands a layout made in a shape in every platform of that shape", () => {
    const previous = resolve({}, "tiktok");
    const next = {
      ...previous,
      sceneClips: previous.sceneClips?.map((clip) => ({
        ...clip,
        preset: "stacked" as const,
      })),
    };
    const written = writtenTimelineEdit({
      context: inFormat("tiktok"),
      edits: {},
      next,
      previous,
      scope: "shape",
      stored: edit,
    });
    expect(written.edit.sceneClips?.[0]?.preset).toBe("split-two-thirds");
    const shown = (id: string) =>
      resolvedTimelineEdit(written.edit, written.edits, {
        base: wide,
        canvas: canvasOf(id),
        context: inFormat(id),
      }).sceneClips?.[0]?.preset;
    expect(shown("tiktok")).toBe("stacked");
    expect(shown("youtube-shorts")).toBe("stacked");
    expect(shown("youtube-1080")).toBe("split-two-thirds");
  });

  it("keeps timing shared wherever it is changed", () => {
    const previous = resolve({}, "tiktok");
    const next = {
      ...previous,
      sceneClips: previous.sceneClips?.map((clip) => ({
        ...clip,
        endMs: 5_000,
      })),
    };
    const written = writtenTimelineEdit({
      context: inFormat("tiktok"),
      edits: {},
      next,
      previous,
      scope: "shape",
      stored: edit,
    });
    expect(written.edit.sceneClips?.[0]?.endMs).toBe(5_000);
  });

  it("keeps a shape laid out apart when the main shape's layout changes", () => {
    const edits: FormatEdits = {
      scenes: { scene: { layouts: { vertical: { preset: "stacked" } } } },
    };
    const previous = resolve(edits, "youtube-1080");
    const written = writtenTimelineEdit({
      context: inFormat("youtube-1080"),
      edits,
      next: {
        ...previous,
        sceneClips: previous.sceneClips?.map((clip) => ({
          ...clip,
          preset: "full" as const,
        })),
      },
      previous,
      scope: "all",
      stored: edit,
    });
    const shown = (id: string) =>
      resolvedTimelineEdit(written.edit, written.edits, {
        base: wide,
        canvas: canvasOf(id),
        context: inFormat(id),
      }).sceneClips?.[0]?.preset;
    expect(shown("youtube-1080")).toBe("full");
    expect(shown("tiktok")).toBe("stacked");
  });

  it("deletes something from every format, whichever it is deleted in", () => {
    const previous = resolve({}, "tiktok");
    const written = writtenTimelineEdit({
      context: inFormat("tiktok"),
      edits: {},
      next: { ...previous, annotationClips: [] },
      previous,
      scope: "shape",
      stored: edit,
    });
    expect(written.edit.annotationClips).toHaveLength(0);
  });
});
