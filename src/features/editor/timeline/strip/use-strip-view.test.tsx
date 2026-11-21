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

/** A scroll area the strip's view is laid on: its scroll position, and the
 * events it sends, fired by hand. */
function scrollArea() {
  const listeners = new Map<string, (event: unknown) => void>();
  return {
    addEventListener: (type: string, listener: (event: unknown) => void) => {
      listeners.set(type, listener);
    },
    fire: (type: string, event: object) =>
      listeners.get(type)?.({ ...event, preventDefault: () => undefined }),
    getBoundingClientRect: () => ({ left: 0, width: 800 }),
    removeEventListener: (type: string) => {
      listeners.delete(type);
    },
    scroll: () => listeners.get("scroll")?.({}),
    scrollLeft: 0,
  };
}

/** The strip's view of a 16 minute recording in an 800 px scroll area,
 * laid on `area`, and how many times it has been drawn. */
function laidOn(area: ReturnType<typeof scrollArea>) {
  const playhead = createPlayhead();
  const held: { view: ReturnType<typeof useStripView> | null } = {
    view: null,
  };
  let drawn = 0;
  function Reader() {
    drawn += 1;
    held.view = useStripView({
      duration: 960,
      focus: null,
      isPlaying: false,
      playhead,
      takenOut: 0,
      width: 800,
    });
    return null;
  }
  const root = createRoot(container());
  act(() => {
    root.render(<Reader />);
  });
  act(() => {
    held.view?.onInitialized(area as unknown as HTMLElement);
  });
  return { held, renders: () => drawn, root };
}

beforeEach(() => {
  // React only flushes `act` work where it is told it runs under tests.
  vi.stubGlobal("IS_REACT_ACT_ENVIRONMENT", true);
  // The strip's zoom waits for a frame; none comes in these tests.
  vi.stubGlobal("requestAnimationFrame", () => 0);
  vi.stubGlobal("cancelAnimationFrame", () => undefined);
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

  it("follows a pan that scrolls on before the strip has drawn", () => {
    const area = scrollArea();
    const { root } = laidOn(area);
    // A swipe scrolls on faster than the strip draws: by the time the view
    // from one scroll event is drawn, the area has scrolled further, and
    // that scroll's own event is still to come.
    act(() => {
      area.scrollLeft = 1_000;
      area.scroll();
      area.scrollLeft = 1_400;
    });
    expect(area.scrollLeft).toBe(1_400);
    act(() => {
      root.unmount();
    });
  });

  it("pans without drawing the strip again until it nears what is drawn", () => {
    const area = scrollArea();
    const { held, renders, root } = laidOn(area);
    const drawn = renders();
    const trackWidth = held.view?.trackWidth ?? 0;
    act(() => {
      for (const left of [100, 200, 300]) {
        area.scrollLeft = left;
        area.scroll();
      }
    });
    expect(renders()).toBe(drawn);
    expect(held.view?.trackWidth).toBe(trackWidth);
    expect(held.view?.now.get().start).toBeCloseTo(300 / trackWidth, 9);
    // Past half a view, what is drawn moves round the view, once.
    act(() => {
      for (const left of [1_100, 1_150]) {
        area.scrollLeft = left;
        area.scroll();
      }
    });
    expect(renders()).toBe(drawn + 1);
    const near = held.view?.near ?? { end: 0, start: 0 };
    const now = held.view?.now.get() ?? { end: 0, start: 0 };
    expect(near.start).toBeLessThan(now.start);
    expect(near.end).toBeGreaterThan(now.end);
    act(() => {
      root.unmount();
    });
  });

  it("scrolls to a view the minimap moves it to", () => {
    const area = scrollArea();
    const { held, root } = laidOn(area);
    const trackWidth = held.view?.trackWidth ?? 0;
    const shown = held.view?.now.get() ?? { end: 0, start: 0 };
    const span = shown.end - shown.start;
    act(() => {
      held.view?.set({ end: 0.5 + span, start: 0.5 });
    });
    expect(area.scrollLeft).toBeCloseTo(0.5 * trackWidth, 3);
    act(() => {
      root.unmount();
    });
  });

  it("zooms in as a pinch spreads, telling the pinch once", () => {
    // A frame comes at once, so the zoom lands within the pinch's own act.
    vi.stubGlobal("requestAnimationFrame", (draw: () => void) => {
      draw();
      return 1;
    });
    const area = scrollArea();
    const { held, root } = laidOn(area);
    const before = held.view?.now.get() ?? { end: 0, start: 0 };
    act(() => {
      area.fire("gesturestart", { clientX: 400, scale: 1 });
      area.fire("gesturechange", { clientX: 400, scale: 2 });
      // The same pinch, as WebKit may tell it a second way.
      area.fire("wheel", { clientX: 400, ctrlKey: true, deltaY: -20 });
      area.fire("gestureend", { clientX: 400, scale: 2 });
    });
    const after = held.view?.now.get() ?? { end: 0, start: 0 };
    // Spread to twice the distance, the strip zooms in eightfold.
    expect((after.end - after.start) / (before.end - before.start)).toBeCloseTo(
      1 / 8,
      6,
    );
    act(() => {
      root.unmount();
    });
  });
});
