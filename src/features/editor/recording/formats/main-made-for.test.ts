// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { defaultCameraOverlay } from "../../export/recording-export-settings";
import { defaultRecordingOutput } from "../../screenshot/screenshot-output";

import { madeForOf, withMainMadeFor } from "./main-made-for";
import { addedFormat, switchedFormat } from "./recording-formats";

import type { FormatDocument } from "./format-document";

const main: FormatDocument = {
  cameraOverlay: defaultCameraOverlay(),
  recordingOutput: defaultRecordingOutput({
    primary: { height: 1_080, width: 1_920 },
  }),
};

describe("main canvas made for a platform", () => {
  it("takes the template's size and keeps clear of its app", () => {
    const made = withMainMadeFor(null, main, "tiktok");
    const primary = made?.document.recordingOutput.primary;
    expect([primary?.width, primary?.height]).toEqual([1_080, 1_920]);
    expect(primary?.clearArea?.bottom).toBe(0.25);
    expect(
      madeForOf(made?.formats ?? null, { height: 1_920, width: 1_080 }),
    ).toBe("tiktok");
  });

  it("takes over a platform of the same template added beside it", () => {
    const added = addedFormat(null, main, {
      preset: "tiktok",
      size: { height: 1_920, width: 1_080 },
    });
    const onMain = switchedFormat(
      added.formats,
      added.document,
      added.formats.list[0]?.id ?? "",
    );
    const made = onMain
      ? withMainMadeFor(onMain.formats, onMain.document, "tiktok")
      : null;
    expect(made?.formats?.list).toHaveLength(1);
    expect(made?.formats?.list[0]?.id).toBe(added.formats.list[0]?.id);
  });

  it("is made for none once its size is typed in", () => {
    const made = withMainMadeFor(null, main, "tiktok");
    expect(
      madeForOf(made?.formats ?? null, { height: 1_080, width: 1_920 }),
    ).toBeNull();
    const custom = made
      ? withMainMadeFor(made.formats, made.document, null)
      : null;
    expect(custom?.document.recordingOutput.primary.clearArea).toBeUndefined();
    expect(custom?.formats?.list[0]?.madeFor).toBeUndefined();
  });
});
