// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import {
  annotationDeleteTargets,
  chosenAnnotationIds,
  toggledAnnotationIds,
} from "./annotation-selection";

const ids = ["a", "b", "c"];
const anyGroupable = () => true;
const unpinned = (id: string) => id !== "pinned";

describe("annotationDeleteTargets", () => {
  it("takes the annotation the hand is pointing at over the chosen ones", () => {
    expect(annotationDeleteTargets(ids, "c", new Set(["a", "b"]))).toEqual(
      new Set(["c"]),
    );
  });

  it("takes every chosen annotation when the hand points at one of them", () => {
    expect(annotationDeleteTargets(ids, "a", new Set(["a", "b"]))).toEqual(
      new Set(["a", "b"]),
    );
  });

  it("ignores a hover on an annotation this layer no longer carries", () => {
    expect(annotationDeleteTargets(ids, "gone", new Set(["a"]))).toEqual(
      new Set(["a"]),
    );
  });

  it("has nothing to delete when neither names an annotation", () => {
    expect(annotationDeleteTargets(ids, null, new Set()).size).toBe(0);
    expect(annotationDeleteTargets([], "a", new Set(["b"])).size).toBe(0);
  });
});

describe("chosenAnnotationIds", () => {
  it("lets go of annotations that are gone", () => {
    expect(
      chosenAnnotationIds(ids, new Set(["a", "gone"]), anyGroupable),
    ).toEqual(new Set(["a"]));
  });

  it("keeps a pinned annotation chosen on its own", () => {
    const chosen = new Set(["pinned"]);
    expect(chosenAnnotationIds(["pinned"], chosen, unpinned)).toBe(chosen);
  });

  it("never groups a pinned annotation with others", () => {
    expect(
      chosenAnnotationIds(
        [...ids, "pinned"],
        new Set(["a", "pinned", "b"]),
        unpinned,
      ),
    ).toEqual(new Set(["a", "b"]));
  });
});

describe("toggledAnnotationIds", () => {
  it("cannot add a pinned annotation to a group", () => {
    const current = new Set(["a", "b"]);
    expect(
      toggledAnnotationIds(current, "pinned", {
        groupable: unpinned,
        toggle: true,
      }),
    ).toBe(current);
  });

  it("chooses a pinned annotation alone on an ordinary click", () => {
    expect(
      toggledAnnotationIds(new Set(["a", "b"]), "pinned", {
        groupable: unpinned,
        toggle: false,
      }),
    ).toEqual(new Set(["pinned"]));
  });
});
