// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { glideGesture, type GlideStroke } from "./glide-gesture-harness";

type Gesture = ReturnType<typeof glideGesture>;

/** Arms the pending minimize with a downward first fold. */
const armMinimize = (gesture: Gesture, stroke: GlideStroke = {}) =>
  gesture.move({ deltaY: 45, ...stroke });

describe("GlideDetector pending minimize", () => {
  it.each([[false], [true]])(
    "arms on a downward first fold (thirds %s)",
    (thirds) => {
      const gesture = glideGesture();
      const detection = armMinimize(gesture, { thirds });

      expect(detection).toEqual({
        becameReady: false,
        changed: true,
        pending: "minimize",
        phase: "settling",
        region: null,
      });
      expect(gesture.detector.pending).toBe("minimize");
    },
  );

  it("requires the minimize fold to dominate a diagonal approach", () => {
    const gesture = glideGesture();
    // Short of the corner cone's horizontal threshold, so no diagonal either.
    expect(gesture.move({ deltaX: 42, deltaY: 46 }).pending).toBeNull();
    expect(gesture.detector.region).toBeNull();

    // Straightening out lets the downward travel take the fold after all.
    expect(gesture.move({ deltaY: 20 }).pending).toBe("minimize");
  });

  it("disarms on an up step, leaving fill reachable again", () => {
    const gesture = glideGesture();
    armMinimize(gesture);
    gesture.rest();

    expect(gesture.move({ deltaY: -50 }).pending).toBeNull();
    expect(gesture.detector.region).toBeNull();
    // The disarm rebases, so the fill fold costs its own full travel.
    expect(gesture.flick({ deltaY: -20 })).toBeNull();
    expect(gesture.glide({ deltaY: -30 })).toBe("full screen");
  });

  it("holds the arm through sideways travel that buys no fold", () => {
    const gesture = glideGesture();
    armMinimize(gesture);
    gesture.rest();

    expect(gesture.move({ deltaX: 30 })).toEqual({
      becameReady: false,
      changed: false,
      pending: "minimize",
      phase: "ready",
      region: null,
    });
    // Short sideways travel neither transitions nor spends the ready window.
    expect(gesture.glide({ deltaY: 50 })).toBe("bottom half");
  });

  it.each([
    [false, 2],
    [true, 3],
  ])(
    "converts a second down step into the bottom row (thirds %s)",
    (thirds, gridCols) => {
      const gesture = glideGesture();
      armMinimize(gesture, { thirds });
      gesture.rest();

      const detection = gesture.move({ deltaY: 50, thirds });
      expect(detection.pending).toBeNull();
      expect(detection.region).toEqual({
        colSpan: gridCols,
        colStart: 0,
        gridCols,
        rowSpan: 1,
        rowStart: 1,
      });
      expect(gesture.label).toBe("bottom half");
    },
  );

  it("steps rows as usual once a region exists", () => {
    const gesture = glideGesture();
    gesture.move({ deltaX: -45 });

    expect(gesture.flick({ deltaY: 50 })).toBe("left half, bottom half");
    expect(gesture.detector.pending).toBeNull();
  });

  it("reports a change exactly on arming, disarming and converting", () => {
    const gesture = glideGesture();
    expect(gesture.move({ deltaY: 45 }).changed).toBe(true);
    gesture.rest();
    expect(gesture.move({ deltaY: 10 }).changed).toBe(false);
    expect(gesture.move({ deltaY: -50 }).changed).toBe(true);
    gesture.rest();
    expect(gesture.move({ deltaY: 20 }).changed).toBe(false);

    expect(gesture.move({ deltaY: 30 }).changed).toBe(true);
    gesture.rest();
    expect(gesture.move({ deltaY: 50 }).changed).toBe(true);
    expect(gesture.label).toBe("bottom half");
  });

  it("clears the pending minimize on reset", () => {
    const gesture = glideGesture();
    armMinimize(gesture);

    expect(gesture.detector.reset()).toEqual({
      becameReady: false,
      changed: true,
      pending: null,
      phase: "ready",
      region: null,
    });
    expect(gesture.detector.pending).toBeNull();
  });
});

describe("GlideDetector minimize re-arm", () => {
  /** Reaches a bottom-edge region, by the ladder or through the arm. */
  const reachBottomRow = (fold?: GlideStroke) => {
    const gesture = glideGesture();
    if (fold) {
      gesture.move(fold);
      gesture.flick({ deltaY: 50, thirds: fold.thirds });
    } else {
      gesture.flick({ deltaY: 50 });
      gesture.flick({ deltaY: 50 });
    }
    return gesture;
  };

  const bottomEdges: [GlideStroke | undefined, string][] = [
    [undefined, "bottom half"],
    [{ deltaX: -45 }, "left half, bottom half"],
    [{ deltaX: 45, thirds: true }, "right third, bottom half"],
  ];

  it.each(bottomEdges)(
    "re-arms from %# with the region retained",
    (fold, expected) => {
      const gesture = reachBottomRow(fold);
      expect(gesture.label).toBe(expected);

      // Down has nowhere left to go on the bottom edge, so the ladder loops
      // back to the minimize instead of no-opping.
      const detection = gesture.flick({ deltaY: 50, thirds: fold?.thirds });
      expect(detection).toBe(expected);
      expect(gesture.detector.pending).toBe("minimize");
    },
  );

  it("holds the arm on a further down step", () => {
    const gesture = reachBottomRow();
    gesture.flick({ deltaY: 50 });

    expect(gesture.flick({ deltaY: 50 })).toBe("bottom half");
    expect(gesture.detector.pending).toBe("minimize");
  });

  it("disarms back to the retained row on an up step", () => {
    const gesture = reachBottomRow({ deltaX: -45 });
    gesture.flick({ deltaY: 50 });
    gesture.rest();

    const detection = gesture.move({ deltaY: -50 });
    expect(detection.pending).toBeNull();
    expect(detection.changed).toBe(true);
    expect(gesture.label).toBe("left half, bottom half");
  });

  it("leaves the row alone until the down step is paid for", () => {
    const gesture = reachBottomRow();
    gesture.rest();

    // A row's own vertical steps stay at the cheap release cost (20).
    expect(gesture.move({ deltaY: 12 }).pending).toBeNull();
    expect(gesture.move({ deltaY: 12 }).pending).toBe("minimize");
    expect(gesture.label).toBe("bottom half");
  });
});

describe("GlideDetector sideways escape from the arm", () => {
  /** The bottom-left corner with the minimize armed over it. */
  const armOverCorner = () => {
    const gesture = glideGesture();
    gesture.move({ deltaX: -45 });
    gesture.flick({ deltaY: 50 });
    gesture.flick({ deltaY: 50 });
    expect(gesture.detector.pending).toBe("minimize");
    return gesture;
  };

  it("steps the retained region sideways and drops the arm", () => {
    const gesture = armOverCorner();
    gesture.rest();

    const detection = gesture.move({ deltaX: 60 });
    expect(detection.pending).toBeNull();
    // The corner's own row carries across, exactly as it would unarmed.
    expect(gesture.label).toBe("bottom half");
  });

  it.each([
    [false, "right half"],
    [true, "right third"],
  ])(
    "takes the sideways first fold with no region (thirds %s)",
    (thirds, expected) => {
      const gesture = glideGesture();
      armMinimize(gesture, { thirds });
      gesture.rest();

      const detection = gesture.move({ deltaX: 60, thirds });
      expect(detection.pending).toBeNull();
      expect(gesture.label).toBe(expected);
    },
  );

  it("spends the window on the escape: one change, then settling", () => {
    const gesture = armOverCorner();
    gesture.rest();

    const detection = gesture.move({ deltaX: 60 });
    expect(detection.changed).toBe(true);
    expect(detection.phase).toBe("settling");
    // Whatever the same flick had left over buys nothing more.
    expect(gesture.move({ deltaX: 60 }).changed).toBe(false);
    expect(gesture.label).toBe("bottom half");
  });
});
