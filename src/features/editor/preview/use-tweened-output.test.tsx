// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { act, useEffect } from "react";
import { createRoot } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { askPictureTween } from "./picture-tween";
import { useTweenedOutput } from "./use-tweened-output";

// The hook is driven through a real React root, as the editor drives it,
// with what the native preview is handed recorded the way it is handed:
// from a passive effect of the component that reads the hook. The root
// renders nothing, so a stand-in for its container is all the DOM it needs.

type Look = { inset: number };

const lookBetween = (from: Look, to: Look, share: number): Look => ({
  inset: from.inset + (to.inset - from.inset) * share,
});

let now = 0;
let frames: FrameRequestCallback[] = [];

/** Runs the animation frames waiting, `ms` after the last ones. */
const advance = (ms: number) => {
  now += ms;
  const waiting = frames;
  frames = [];
  act(() => {
    for (const frame of waiting) frame(now);
  });
};

/** A stand-in for the root's container, with what React DOM reads of one:
 * the root renders nothing into it. */
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

/** Mounts a reader of the hook whose drawn looks, as a passive effect hands
 * them on, land in `handed`. */
function mount(first: Look, freshEachRender = false) {
  const handed: number[] = [];
  let renders = 0;
  function Reader({ target }: { target: Look }) {
    renders += 1;
    const drawn = useTweenedOutput(
      freshEachRender ? { ...target } : target,
      { part: "picture", workspace: "screenshot" },
      lookBetween,
    );
    useHandOn(drawn, handed);
    return null;
  }
  const root = createRoot(container());
  const render = (target: Look) => {
    act(() => {
      root.render(<Reader target={target} />);
    });
  };
  render(first);
  return {
    handed,
    render,
    renders: () => renders,
    unmount: () => {
      act(() => {
        root.unmount();
      });
    },
  };
}

function useHandOn(drawn: Look, handed: number[]) {
  useEffect(() => {
    handed.push(drawn.inset);
  }, [drawn, handed]);
}

beforeEach(() => {
  // React only flushes `act` work where it is told it runs under tests.
  vi.stubGlobal("IS_REACT_ACT_ENVIRONMENT", true);
  // What React DOM and the reduced-motion query read of the window.
  vi.stubGlobal("window", {
    // Nothing is focused, so no frame is ever one of these.
    HTMLIFrameElement: Object,
    document: { activeElement: null },
    event: undefined,
    matchMedia: () => ({
      addEventListener: () => undefined,
      matches: false,
      removeEventListener: () => undefined,
    }),
  });
  now = 0;
  frames = [];
  vi.spyOn(performance, "now").mockImplementation(() => now);
  vi.stubGlobal("requestAnimationFrame", (frame: FrameRequestCallback) => {
    frames.push(frame);
    return frames.length;
  });
  vi.stubGlobal("cancelAnimationFrame", () => {
    frames = [];
  });
});

afterEach(() => {
  vi.unstubAllGlobals();
  vi.restoreAllMocks();
});

describe("useTweenedOutput", () => {
  it("hands on the old look first when an island asks for a change", () => {
    const view = mount({ inset: 0 });
    askPictureTween("screenshot");
    view.render({ inset: 100 });
    // Nothing the preview was handed after the change is the new look until
    // the tween reaches it, and the first is still the old one.
    const afterChange = view.handed.slice(1);
    expect(afterChange[0]).toBe(0);
    expect(afterChange.every((inset) => inset < 100)).toBe(true);
    view.unmount();
  });

  it("moves to the new look and settles on it", () => {
    const view = mount({ inset: 0 });
    askPictureTween("screenshot");
    view.render({ inset: 100 });
    for (let step = 0; step < 30; step += 1) advance(16);
    const after = view.handed.slice(1);
    expect(after[after.length - 1]).toBe(100);
    for (let index = 1; index < after.length; index += 1)
      expect(after[index]).toBeGreaterThanOrEqual(after[index - 1]);
    expect(after.some((inset) => inset > 0 && inset < 100)).toBe(true);
    expect(frames).toHaveLength(0);
    view.unmount();
  });

  it("jumps to a change nobody asked to move to", () => {
    const view = mount({ inset: 0 });
    view.render({ inset: 100 });
    expect(view.handed).toEqual([0, 100]);
    view.unmount();
  });

  it("settles when the target is built afresh on every render", () => {
    const view = mount({ inset: 10 }, true);
    const before = view.renders();
    view.render({ inset: 10 });
    view.render({ inset: 20 });
    // One render per `render` call (two under the tween's first step at
    // most): a hook that set state on every render would never return.
    expect(view.renders() - before).toBeLessThanOrEqual(4);
    expect(view.handed[view.handed.length - 1]).toBe(20);
    view.unmount();
  });
});
