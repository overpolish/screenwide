// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it, vi } from "vitest";

import {
  afterPreviewLayout,
  fitPreviewDuringResize,
  ResizeFit,
  resizeFitWidth,
} from "./use-native-preview-fit";

const deferred = () => {
  let resolve: () => void = () => undefined;
  const promise = new Promise<void>((done) => {
    resolve = done;
  });
  return { promise, resolve };
};

describe("native fit ordering", () => {
  it("waits for the latest coalesced resize layout before fitting", async () => {
    const first = deferred();
    const latest = deferred();
    const layout = { current: first.promise };
    const fit = vi.fn().mockResolvedValue(undefined);
    const fitting = afterPreviewLayout(layout, fit);
    expect(fit).not.toHaveBeenCalled();
    layout.current = latest.promise;
    first.resolve();
    await first.promise;
    expect(fit).not.toHaveBeenCalled();
    latest.resolve();
    await fitting;
    expect(fit).toHaveBeenCalledOnce();
  });

  it("fits immediately after an already settled layout", async () => {
    const fit = vi.fn().mockResolvedValue(undefined);
    await afterPreviewLayout({ current: Promise.resolve() }, fit);
    expect(fit).toHaveBeenCalledOnce();
  });
});

describe("tool-panel resize fit", () => {
  it("includes the final fit in every resize layout, until coalesced layouts finish", async () => {
    const growth = deferred();
    const first = deferred();
    const last = deferred();
    const fitRef = { current: null as ResizeFit | null };
    const layoutRef = { current: first.promise };
    const measure = vi.fn();
    const resizing = fitPreviewDuringResize({
      fitRef,
      gutter: 320,
      isCurrent: () => true,
      layoutRef,
      measure,
      resize: () => {
        expect(resizeFitWidth(fitRef.current, 800, 1)).toBe(480);
        return growth.promise;
      },
      sessionId: 1,
    });
    expect(resizeFitWidth(fitRef.current, 1120, 1)).toBe(800);
    // A display that refuses the requested growth uses its actual bounds.
    expect(resizeFitWidth(fitRef.current, 950, 1)).toBe(630);
    growth.resolve();
    await growth.promise;
    expect(measure).toHaveBeenCalledOnce();
    layoutRef.current = last.promise;
    first.resolve();
    await first.promise;
    expect(fitRef.current).not.toBeNull();
    last.resolve();
    await resizing;
    expect(fitRef.current).toBeNull();
    expect(resizeFitWidth(fitRef.current, 1120, 1)).toBeUndefined();
  });

  it("requests a fresh fit even when a later panel opening cannot grow the window", async () => {
    const fitRef = { current: null as ResizeFit | null };
    const revisions: number[] = [];
    const options = {
      fitRef,
      gutter: 320,
      isCurrent: () => true,
      layoutRef: { current: Promise.resolve() },
      measure: () => {
        if (fitRef.current) revisions.push(fitRef.current.revision);
      },
      resize: () => Promise.resolve(),
      sessionId: 1,
    };
    await fitPreviewDuringResize(options);
    await fitPreviewDuringResize(options);
    expect(revisions).toHaveLength(2);
    expect(revisions[0]).not.toBe(revisions[1]);
  });

  it("clears a failed resize and never applies an old session's fit to a new recording", async () => {
    const fitRef = { current: null as ResizeFit | null };
    const measure = vi.fn();
    await expect(
      fitPreviewDuringResize({
        fitRef,
        gutter: 320,
        isCurrent: () => false,
        layoutRef: { current: Promise.resolve() },
        measure,
        resize: () => {
          expect(resizeFitWidth(fitRef.current, 800, 2)).toBeUndefined();
          return Promise.reject(new Error("resize failed"));
        },
        sessionId: 1,
      }),
    ).rejects.toThrow("resize failed");
    expect(fitRef.current).toBeNull();
    expect(measure).not.toHaveBeenCalled();
  });
});
