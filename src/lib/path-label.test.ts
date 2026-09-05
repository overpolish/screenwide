// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { truncatePath } from "./path-label";

describe("path labels", () => {
  it("leaves short paths unchanged", () => {
    for (const path of ["/", "C:\\", "/Users/dom", "file.txt", ""])
      expect(truncatePath(path)).toBe(path);
  });
  it("keeps root and trailing segments with native separators", () => {
    expect(truncatePath("/Users/dom/Documents/2026/August", 25)).toBe(
      "/Users/.../2026/August",
    );
    expect(truncatePath("C:\\Users\\dom\\Documents\\Exports", 25)).toBe(
      "C:\\...\\Documents\\Exports",
    );
  });
  it("preserves a long filename extension", () => {
    const result = truncatePath(
      "/Users/dom/An extremely long name.screenwide",
      20,
      "file",
    );
    expect(result).toHaveLength(20);
    expect(result.endsWith(".screenwide")).toBe(true);
    expect(result).toContain("...");
  });
  it("keeps UNC server and share together", () => {
    const result = truncatePath(
      "\\\\server\\share\\Documents\\Recordings\\Final",
      30,
    );
    expect(result.startsWith("\\\\server\\share\\...")).toBe(true);
    expect(result.endsWith("Final")).toBe(true);
    expect(truncatePath("\\\\longservername\\longsharename", 10)).toBe(
      "\\\\longservername\\longsharename",
    );
  });
  it("handles long segments and rejects invalid budgets", () => {
    expect(truncatePath("abcdefghijklmnopqrstuvwxyz", 10)).toBe("abcd...xyz");
    expect(() => truncatePath("abc", 0)).toThrow(RangeError);
  });
});
