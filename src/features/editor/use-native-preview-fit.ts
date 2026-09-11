// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { RefObject, useCallback } from "react";

/** Wait through coalesced layouts too: completing one IPC can start the next. */
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
  sessionIdRef,
  setBasis,
  startedRef,
}: {
  layoutRef: RefObject<Promise<unknown>>;
  measureRef: RefObject<() => void>;
  reset: (sessionId: number, fitWidth?: number) => Promise<unknown>;
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

  return { fitPreview, setFitBasis };
}
