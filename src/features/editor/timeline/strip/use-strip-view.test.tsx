// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { act } from "react";
import { createRoot } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { createPlayhead } from "../scrub-playhead";

import { useStripView } from "./use-strip-view";

// The hook is driven through a real React root that renders nothing, so a
// stand-in for its container is all the DOM it needs.

const container = () =>
  ({
    addEventListener: () => undefined,
    namespaceURI: "http://www.w3.org/1999/xhtml",
    nodeName: "DIV",
    nodeType: 1,
    ownerDocument: { addEventListener: () => undefined },
    removeEventListener: () => undefined,
    tagName: "DIV",
  }) as unknown as Element;

/** How wide the track is drawn for a strip `duration` seconds long, after
 * each of `durations` in turn, in an 800 px scroll area. */
function trackWidths(durations: number[]) {
  const widths: number[] = [];
  const playhead = createPlayhead();
  function Reader({ duration }: { duration: number }) {
    const view = useStripView({
      duration,
      focus: null,
      isPlaying: false,
      playhead,
      takenOut: 0,
      width: 800,
    });
    widths.push(view.trackWidth);
    return null;
  }
  const root = createRoot(container());
  for (const duration of durations)
    act(() => {
      root.render(<Reader duration={duration} />);
    });
  act(() => {
    root.unmount();
  });
  return widths;
}

beforeEach(() => {
  // React only flushes `act` work where it is told it runs under tests.
  vi.stubGlobal("IS_REACT_ACT_ENVIRONMENT", true);
  vi.stubGlobal("window", {
    HTMLIFrameElement: Object,
    document: { activeElement: null },
    event: undefined,
  });
});

afterEach(() => {
  vi.unstubAllGlobals();
});

describe("the strip's view", () => {
  it("takes back the room a gap taken away leaves past its end", () => {
    // 20 s fits whole; 5 s of gap taken away leaves 15 s to fill it.
    const widths = trackWidths([20, 15]);
    expect(widths[0]).toBe(784);
    expect(widths[widths.length - 1]).toBe(784);
  });
});
