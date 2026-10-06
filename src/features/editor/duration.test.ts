// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { formatBytes } from "./duration";

describe("formatBytes", () => {
  it("reads as Finder shows it on macOS", () => {
    expect(formatBytes(10_512_171, 1000)).toBe("10.5 MB");
    expect(formatBytes(148_000_000, 1000)).toBe("148 MB");
    expect(formatBytes(75_312, 1000)).toBe("75 KB");
    expect(formatBytes(2_412_000_000, 1000)).toBe("2.41 GB");
    expect(formatBytes(512, 1000)).toBe("512 bytes");
  });

  it("reads as Explorer shows it on Windows", () => {
    expect(formatBytes(10_512_171, 1024)).toBe("10 MB");
    expect(formatBytes(1024, 1024)).toBe("1 KB");
  });

  it("moves to the next unit when rounding would reach it", () => {
    expect(formatBytes(999_999, 1000)).toBe("1 MB");
    expect(formatBytes(999_996_000, 1000)).toBe("1 GB");
  });

  it("says when there is no size", () => {
    expect(formatBytes(0, 1000)).toBe("Unknown size");
  });
});
