// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { RefObject, useCallback } from "react";

/** Wait through coalesced layouts too: completing one IPC can start the next.
 */
export async function afterPreviewLayout(
  layout: RefObject<Promise<unknown>>,
  request: () => Promise<unknown>,
) {
  let pending: Promise<unknown>;
  do {
    pending = layout.current;
    await pending;
  } while (pending !== layout.current);
  await request();
}

export type ResizeFit = {
  gutter: number;
  revision: number;
  /** Whether a viewport this wide leaves the picture its room beside the
   * panel. Where it does not, the picture is not fitted beside the panel. */
  roomy: (viewportWidth: number) => boolean;
  sessionId: number;
};
let resizeFitRevision = 0;

export function resizeFitWidth(
  fit: ResizeFit | null,
  viewportWidth: number,
  sessionId: number,
) {
  return fit?.sessionId === sessionId && fit.roomy(viewportWidth)
    ? Math.max(1, viewportWidth - fit.gutter)
    : undefined;
}

/** Mark every resize layout before growth starts, and keep the fit attached
 * through the last coalesced layout. No separate transform can flash later. */
export async function fitPreviewDuringResize({
  fitRef,
  gutter,
  isCurrent,
  layoutRef,
  measure,
  resize,
  roomy = () => true,
  sessionId,
}: {
  fitRef: RefObject<ResizeFit | null>;
  gutter: number;
  isCurrent: () => boolean;
  layoutRef: RefObject<Promise<unknown>>;
  measure: () => void;
  resize: () => Promise<void>;
  sessionId: number;
  roomy?: (viewportWidth: number) => boolean;
}) {
  const fit = { gutter, revision: ++resizeFitRevision, roomy, sessionId };
  fitRef.current = fit;
  try {
    await resize();
    if (isCurrent() && fitRef.current === fit) {
      measure();
      await afterPreviewLayout(layoutRef, () => Promise.resolve());
    }
  } finally {
    if (fitRef.current === fit) fitRef.current = null;
  }
}

/**
 * A one-time native transform must use the post-resize base pane geometry.
 *
 * The fit also leaves its width with the surface as the basis a double-click
 * reset returns to, so `setFitBasis` is only needed to hand that basis back
 * when a tool panel closes: it stores the width and moves nothing.
 */
export function useNativePreviewFit({
  layoutRef,
  measureRef,
  onError,
  reset,
  resizeFitRef,
  sessionIdRef,
  setBasis,
  startedRef,
}: {
  layoutRef: RefObject<Promise<unknown>>;
  measureRef: RefObject<() => void>;
  reset: (sessionId: number, fitWidth?: number) => Promise<unknown>;
  resizeFitRef: RefObject<ResizeFit | null>;
  sessionIdRef: RefObject<number>;
  setBasis: (sessionId: number, fitWidth?: number) => Promise<unknown>;
  startedRef: RefObject<boolean>;
  onError?: (message: string) => void;
}) {
  const fitPreview = useCallback(
    (fitWidth?: number) => {
      if (!startedRef.current) return;
      const sessionId = sessionIdRef.current;
      measureRef.current();
      void afterPreviewLayout(layoutRef, async () => {
        if (startedRef.current && sessionIdRef.current === sessionId)
          await reset(sessionId, fitWidth);
      }).catch((cause: unknown) => {
        onError?.(String(cause));
      });
    },
    [layoutRef, measureRef, onError, reset, sessionIdRef, startedRef],
  );

  const setFitBasis = useCallback(
    (fitWidth?: number) => {
      if (!startedRef.current) return;
      void setBasis(sessionIdRef.current, fitWidth).catch((cause: unknown) => {
        onError?.(String(cause));
      });
    },
    [onError, sessionIdRef, setBasis, startedRef],
  );

  const fitDuringResize = useCallback(
    async (
      gutter: number,
      resize: () => Promise<void>,
      roomy?: (viewportWidth: number) => boolean,
    ) => {
      if (!startedRef.current) return resize();
      const sessionId = sessionIdRef.current;
      await fitPreviewDuringResize({
        fitRef: resizeFitRef,
        gutter,
        isCurrent: () =>
          startedRef.current && sessionIdRef.current === sessionId,
        layoutRef,
        measure: () => {
          measureRef.current();
        },
        resize,
        roomy,
        sessionId,
      });
    },
    [layoutRef, measureRef, resizeFitRef, sessionIdRef, startedRef],
  );

  return { fitDuringResize, fitPreview, setFitBasis };
}
