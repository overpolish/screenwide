// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { folderName } from "./path-label";

describe("folder names", () => {
  it("takes the last part of a path on either platform", () => {
    expect(folderName("/Users/dom/Movies/Recordings")).toBe("Recordings");
    expect(folderName("C:\\Users\\dom\\Videos\\Exports")).toBe("Exports");
    expect(folderName("\\\\server\\share\\Final")).toBe("Final");
  });
  it("ignores a trailing separator", () => {
    expect(folderName("/Users/dom/Movies/")).toBe("Movies");
    expect(folderName("C:\\Exports\\")).toBe("Exports");
  });
  it("shows a root whole, having no name of its own", () => {
    expect(folderName("/")).toBe("/");
    expect(folderName("C:\\")).toBe("C:\\");
    expect(folderName("C:")).toBe("C:");
    expect(folderName("\\\\server\\share")).toBe("\\\\server\\share");
  });
});
