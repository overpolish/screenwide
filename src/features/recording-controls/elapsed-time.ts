// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { RecordingSnapshot } from "./types";

/**
 * Elapsed time is derived from the snapshot's timestamps rather than
 * accumulated tick by tick, so a window that reloads, or opens mid-recording,
 * still shows the same time as every other one.
 */
export const elapsedMilliseconds = (
  snapshot: RecordingSnapshot,
  now: number,
) => {
  const openSpan =
    snapshot.status === "recording" && snapshot.startedAtMs !== null
      ? Math.max(0, now - snapshot.startedAtMs)
      : 0;

  return snapshot.accumulatedMs + openSpan;
};
