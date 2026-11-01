// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import {
  defaultRecordingOutput,
  reopenedRecordingOutput,
  restoredRecordingOutput,
} from "./screenshot-output";

const primary = { height: 1_080, width: 1_920 };

const padded = () => {
  const output = defaultRecordingOutput({ primary });
  return {
    ...output,
    primary: {
      ...output.primary,
      cropHeight: output.primary.cropHeight + 80,
      cropWidth: output.primary.cropWidth + 80,
      cropX: output.primary.cropX - 40,
      cropY: output.primary.cropY - 40,
      recenterInsetColor: "#1e1e1e",
    },
  };
};

describe("reopening a recording's output", () => {
  it("keeps the colour its inset is filled with", () => {
    const saved = padded();
    const reopened = reopenedRecordingOutput({ primary, saved });

    expect(reopened.primary.recenterInsetColor).toBe("#1e1e1e");
    expect(reopened.primary.cropWidth).toBe(saved.primary.cropWidth);
  });

  it("drops it from a look remembered from another recording", () => {
    const restored = restoredRecordingOutput({ persisted: padded(), primary });

    expect(restored.primary.recenterInsetColor).toBeNull();
  });
});
