// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useState } from "react";

import { getRecordingWrittenBytes } from "./api";
import { RecordingStatus } from "./types";

/** macOS writes the recording out every two seconds, so reading more often
 * than once a second would only repeat the same size. */
const POLL_INTERVAL_MS = 1_000;

/**
 * How much of the recording is on disk while one is under way: undefined when
 * the size is not shown, null until the first reading. A paused recording is
 * still read, so a pause before the first reading does not keep the spinner,
 * and whatever the writer lands after the pause still shows.
 */
export function useRecordingWrittenBytes(
  enabled: boolean,
  status: RecordingStatus,
) {
  const isUnderWay = status === "recording" || status === "paused";
  const [bytes, setBytes] = useState<number | null>(null);
  const [wasUnderWay, setWasUnderWay] = useState(isUnderWay);
  // The dock stays mounted between recordings: each one starts from the
  // spinner rather than the size the last one ended on.
  if (wasUnderWay !== isUnderWay) {
    setWasUnderWay(isUnderWay);
    if (!isUnderWay) setBytes(null);
  }

  useEffect(() => {
    if (!enabled || !isUnderWay) return;

    let cancelled = false;
    let timeout = 0;
    // Each reading waits for the last, so a slow disk never stacks requests.
    const read = () => {
      getRecordingWrittenBytes()
        .then((written) => {
          // Zero is a file not yet begun, which the spinner stands for.
          if (!cancelled && written !== null && written > 0) setBytes(written);
        })
        .catch((error: unknown) => {
          console.error("Could not read the recording's size", error);
        })
        .finally(() => {
          if (!cancelled) timeout = window.setTimeout(read, POLL_INTERVAL_MS);
        });
    };
    read();

    return () => {
      cancelled = true;
      window.clearTimeout(timeout);
    };
  }, [enabled, isUnderWay]);

  return enabled && isUnderWay ? bytes : undefined;
}
