// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// TEMPORARY(#12): what the frontend spends on a layout before the native
// command sees it, sent with the layout so the native side writes it into
// the resize log beside its own timing.

export type ResizeTiming = {
  idleMs: number | null;
  measureMs: number | null;
  payloadChars: number;
  previousRoundTripMs: number | null;
  queuedMs: number;
};

/** A layout ready to send: when it was ready and what building it cost. */
export type ReadyLayoutTiming = {
  measureMs: number | null;
  payloadChars: number;
  readyAt: number;
};

export const readyLayoutTiming = (
  payloadChars: number,
  measureStartedAt?: number,
): ReadyLayoutTiming => {
  const readyAt = performance.now();
  return {
    measureMs:
      measureStartedAt === undefined ? null : readyAt - measureStartedAt,
    payloadChars,
    readyAt,
  };
};

/** Times the layouts one queue sends, one in flight at a time. */
export function createResizeTimer() {
  let settledAt: number | null = null;
  let previousRoundTripMs: number | null = null;
  return {
    /** Stamps a layout as it is sent; call the answer when its invoke settles. */
    send: (ready: ReadyLayoutTiming) => {
      const sentAt = performance.now();
      const timing: ResizeTiming = {
        idleMs: settledAt === null ? null : sentAt - settledAt,
        measureMs: ready.measureMs,
        payloadChars: ready.payloadChars,
        previousRoundTripMs,
        queuedMs: sentAt - ready.readyAt,
      };
      const settled = () => {
        settledAt = performance.now();
        previousRoundTripMs = settledAt - sentAt;
      };
      return { settled, timing };
    },
  };
}
