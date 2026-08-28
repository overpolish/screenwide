// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { getNextGridItemIndex } from "./button-grid-navigation";

describe("getNextGridItemIndex", () => {
  it("moves spatially through a complete grid", () => {
    expect(
      getNextGridItemIndex({
        columns: 3,
        currentIndex: 4,
        direction: "left",
        itemCount: 9,
      }),
    ).toBe(3);
    expect(
      getNextGridItemIndex({
        columns: 3,
        currentIndex: 4,
        direction: "right",
        itemCount: 9,
      }),
    ).toBe(5);
    expect(
      getNextGridItemIndex({
        columns: 3,
        currentIndex: 4,
        direction: "up",
        itemCount: 9,
      }),
    ).toBe(1);
    expect(
      getNextGridItemIndex({
        columns: 3,
        currentIndex: 4,
        direction: "down",
        itemCount: 9,
      }),
    ).toBe(7);
  });

  it("stops at row and grid edges", () => {
    expect(
      getNextGridItemIndex({
        columns: 3,
        currentIndex: 3,
        direction: "left",
        itemCount: 9,
      }),
    ).toBe(3);
    expect(
      getNextGridItemIndex({
        columns: 3,
        currentIndex: 5,
        direction: "right",
        itemCount: 9,
      }),
    ).toBe(5);
    expect(
      getNextGridItemIndex({
        columns: 3,
        currentIndex: 1,
        direction: "up",
        itemCount: 9,
      }),
    ).toBe(1);
    expect(
      getNextGridItemIndex({
        columns: 3,
        currentIndex: 7,
        direction: "down",
        itemCount: 9,
      }),
    ).toBe(7);
  });

  it("does not jump columns in a partial final row", () => {
    expect(
      getNextGridItemIndex({
        columns: 3,
        currentIndex: 4,
        direction: "down",
        itemCount: 5,
      }),
    ).toBe(4);
    expect(
      getNextGridItemIndex({
        columns: 3,
        currentIndex: 3,
        direction: "right",
        itemCount: 5,
      }),
    ).toBe(4);
    expect(
      getNextGridItemIndex({
        columns: 3,
        currentIndex: 4,
        direction: "right",
        itemCount: 5,
      }),
    ).toBe(4);
  });
});
