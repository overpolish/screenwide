// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { GlideDetector, type GlideDetectorOptions } from "./glide-detection";
import { describeRegion, type GlideRegion } from "./glide-regions";

/** One test sample: whatever it leaves out did not move. */
export type GlideStroke = {
  deltaX?: number;
  deltaY?: number;
  thirds?: boolean;
};

const label = (region: GlideRegion | null) => region && describeRegion(region);

/**
 * A detector on a test clock, so gestures read the way they are performed:
 * `flick` is one sample after the fingers have rested - the unit gesture that
 * buys a single transition - while `move` keeps feeding the same window.
 */
export const glideGesture = (options: Partial<GlideDetectorOptions> = {}) => {
  const detector = new GlideDetector(options);
  let now = 0;

  const advance = (ms: number) => {
    now += ms;
  };
  const settle = () => detector.settle(now).becameReady;
  const move = (stroke: GlideStroke = {}) =>
    detector.update({
      deltaX: stroke.deltaX ?? 0,
      deltaY: stroke.deltaY ?? 0,
      thirds: stroke.thirds ?? false,
      timestamp: now,
    });
  const rest = (ms = detector.options.restMs) => {
    advance(ms);
    return settle();
  };

  return {
    /** Moves the clock on with the fingers down and still. */
    advance,
    detector,
    /** Lets the rest complete, then plays one sample: the unit gesture. */
    flick: (stroke?: GlideStroke) => {
      rest();
      return label(move(stroke).region);
    },
    /** Plays one sample in the current window and names where it lands. */
    glide: (stroke?: GlideStroke) => label(move(stroke).region),
    get label() {
      return label(detector.region);
    },
    move,
    /** Advances past the rest and completes it, as the UI timer would. */
    rest,
    /** Completes the rest at the current time, as the UI timer would. */
    settle,
  };
};
