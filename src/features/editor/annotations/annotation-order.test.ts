// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import {
  arrangedGroup,
  availableArrangements,
  availableGroupArrangements,
  arranged,
  freshAnnotationIndex,
} from "./annotation-order";

describe("freshAnnotationIndex", () => {
  const drawn = ["redact", "highlight", "spotlight", "arrow", "text"] as const;

  it("puts a highlight under the spotlights and everything drawn over the page", () => {
    expect(freshAnnotationIndex(drawn, "highlight")).toBe(2);
  });

  it("puts a spotlight over the highlights and under the rest", () => {
    expect(freshAnnotationIndex(drawn, "spotlight")).toBe(3);
  });

  it("puts anything else on top", () => {
    expect(freshAnnotationIndex(drawn, "counter")).toBe(drawn.length);
  });
});

// Items as [id, start, end]: two meet when their spans overlap.
type Span = [string, number, number];
const meets = (a: Span, b: Span) => a[1] < b[2] && b[1] < a[2];
const ids = (items: Span[]) => items.map(([id]) => id);

describe("arranged", () => {
  const items: Span[] = [
    ["a", 0, 10],
    ["b", 20, 30],
    ["c", 5, 25],
    ["d", 0, 10],
  ];

  it("brings an item over the next one it meets, passing those it never does", () => {
    expect(ids(arranged(items, 0, { arrangement: "forward", meets }))).toEqual([
      "b",
      "c",
      "a",
      "d",
    ]);
  });

  it("sends an item under the previous one it meets", () => {
    expect(
      ids(
        arranged(items, 2, {
          arrangement: "backward",
          meets,
        }),
      ),
    ).toEqual(["a", "c", "b", "d"]);
  });

  it("moves to the very top or bottom", () => {
    expect(ids(arranged(items, 2, { arrangement: "back", meets }))).toEqual([
      "c",
      "a",
      "b",
      "d",
    ]);
    expect(ids(arranged(items, 0, { arrangement: "front", meets }))).toEqual([
      "b",
      "c",
      "d",
      "a",
    ]);
  });

  it("offers no move, and makes none, past items it never meets", () => {
    expect(availableArrangements(items, 1, meets)).toEqual({
      canBringForward: true,
      canSendBackward: false,
    });
    expect(arranged(items, 1, { arrangement: "backward", meets })).toBe(items);
    expect(arranged(items, 1, { arrangement: "back", meets })).toBe(items);
  });
});

describe("arrangedGroup", () => {
  const always = () => true;
  const letters = (list: string[]) => list.join("");
  const group =
    (members: string) =>
    (item: string): boolean =>
      members.includes(item);

  it("steps the group over the next item, keeping its own stacking", () => {
    expect(
      letters(
        arrangedGroup(["a", "x", "b", "y"], group("ab"), {
          arrangement: "forward",
          meets: always,
        }),
      ),
    ).toBe("xayb");
    expect(
      letters(
        arrangedGroup(["x", "a", "y", "b"], group("ab"), {
          arrangement: "backward",
          meets: always,
        }),
      ),
    ).toBe("axby");
  });

  it("holds a member back behind another member rather than passing it", () => {
    // `a` never meets `x`, so once `b` has passed it the next item `a` meets
    // is `b` itself.
    const spans: Span[] = [
      ["a", 0, 10],
      ["b", 5, 25],
      ["x", 15, 30],
    ];
    expect(
      ids(
        arrangedGroup(spans, ([id]) => id !== "x", {
          arrangement: "forward",
          meets,
        }),
      ),
    ).toEqual(["a", "x", "b"]);
  });

  it("takes the group to the very top or bottom as one block", () => {
    const items = ["b", "x", "a", "y"];
    expect(
      letters(
        arrangedGroup(items, group("ab"), {
          arrangement: "front",
          meets: always,
        }),
      ),
    ).toBe("xyba");
    expect(
      letters(
        arrangedGroup(items, group("ab"), {
          arrangement: "back",
          meets: always,
        }),
      ),
    ).toBe("baxy");
  });

  it("offers no move, and makes none, past items no member meets", () => {
    const spans: Span[] = [
      ["a", 0, 10],
      ["b", 0, 10],
      ["x", 20, 30],
    ];
    const isMember = ([id]: Span) => id === "a" || id === "b";
    expect(availableGroupArrangements(spans, isMember, meets)).toEqual({
      canBringForward: false,
      canSendBackward: false,
    });
    expect(
      arrangedGroup(spans, isMember, { arrangement: "front", meets }),
    ).toBe(spans);
  });
});
