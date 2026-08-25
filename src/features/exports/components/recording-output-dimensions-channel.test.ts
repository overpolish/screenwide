// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it, vi } from "vitest";

import { createRecordingOutputDimensionsChannel } from "./recording-output-dimensions-channel";

describe("recording output dimensions channel", () => {
  it("publishes only changed dimensions and retains the final frame", () => {
    const channel = createRecordingOutputDimensionsChannel();
    const listener = vi.fn();
    channel.subscribe(listener);

    channel.publish({ height: 1080, width: 1920 });
    channel.publish({ height: 1080, width: 1920 });
    channel.publish({ height: 1080, width: 1200 });

    expect(listener).toHaveBeenCalledTimes(2);
    expect(channel.getSnapshot()).toEqual({ height: 1080, width: 1200 });
  });
});
