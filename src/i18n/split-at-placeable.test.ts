// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { splitAtPlaceable, splitAtPlaceables } from "./split-at-placeable";

describe("splitting a message round its elements", () => {
  it("keeps the elements where the language reads them, reordered too", () => {
    const parts = splitAtPlaceables(
      2,
      ([first, second]) => `${second} then \u2068${first}\u2069.`,
    );
    expect(parts).toEqual([1, " then ", 0, "."]);
  });

  it("gives the text either side of a single element", () => {
    expect(splitAtPlaceable((key) => `Hold ${key} to box.`)).toEqual([
      "Hold ",
      " to box.",
    ]);
    expect(splitAtPlaceable(() => "No key here.")).toEqual([
      "No key here.",
      "",
    ]);
  });
});
