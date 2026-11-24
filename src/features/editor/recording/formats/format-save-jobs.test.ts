// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { defaultCameraOverlay } from "../../export/recording-export-settings";
import { defaultRecordingOutput } from "../../screenshot/screenshot-output";

import { formatSaveJobs } from "./format-save-jobs";

import type { FormatDocument } from "./format-document";
import type { FrameFormats } from "../../../../bindings/FrameFormats";
import type { RecordingTimelineEdit } from "../../timeline/editing/recording-timeline-edit";

const edit = { artifactId: 1 } as unknown as RecordingTimelineEdit;

const document: FormatDocument = {
  cameraOverlay: defaultCameraOverlay(),
  recordingOutput: defaultRecordingOutput({
    primary: { height: 1_800, width: 3_200 },
  }),
};

const sizesOf = (formats: FrameFormats) =>
  formatSaveJobs({
    document,
    edit,
    fileStem: "Demo",
    formats,
    isAll: true,
    stored: edit,
  }).map(({ isCopy, recordingOutput }) => [
    recordingOutput.primary.width,
    recordingOutput.primary.height,
    isCopy,
  ]);

describe("the files an export writes", () => {
  it("writes a platform at its template's size, its shape at its canvas", () => {
    expect(
      sizesOf({
        active: "wide",
        list: [{ id: "wide", platforms: ["wide", "youtube-1080"] }],
      }),
    ).toEqual([
      [3_200, 1_800, false],
      [1_920, 1_080, false],
    ]);
  });

  it("copies a platform that comes out the same size and showing the same", () => {
    expect(
      sizesOf({
        active: "wide",
        list: [{ id: "wide", platforms: ["youtube-1080", "x-landscape"] }],
      }),
    ).toEqual([
      [1_920, 1_080, false],
      [1_920, 1_080, true],
    ]);
  });
});
