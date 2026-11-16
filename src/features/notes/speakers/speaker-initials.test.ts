// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { speakerInitials } from "./speaker-initials";

describe("a speaker's initials", () => {
  it("are a numbered speaker's number", () => {
    expect(speakerInitials("Speaker 3")).toBe("3");
  });

  it("are the first letters of a name's first two words", () => {
    expect(speakerInitials("priya shah rao")).toBe("PS");
  });

  it("keep a letter written as two code units whole", () => {
    expect(speakerInitials("𝒜da")).toBe("𝒜");
  });
});
