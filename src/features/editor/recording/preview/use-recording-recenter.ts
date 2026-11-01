// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useRef } from "react";

import { useRecenterInsetControls } from "../../recenter-inset-channel";
import { ScreenshotOutputSettings } from "../../screenshot/screenshot-output";
import {
  getRecordingRecenterAnalysis,
  recenterScreenshotContent,
} from "../../screenshot/screenshot-recenter";

/**
 * The recording's content analysis, published for the crop's commit and the
 * Screen island's inset presets and Recenter. Both read the frame under the
 * playhead, so the picture is parked before it is read.
 */
export function useRecordingRecenter({
  artifactId,
  getPositionMs,
  onOutputChange,
  output,
  pause,
  source,
}: {
  artifactId: number;
  getPositionMs: () => number;
  output: ScreenshotOutputSettings;
  pause: () => void;
  onOutputChange?: (settings: ScreenshotOutputSettings) => void;
  source?: { height: number; width: number };
}) {
  const requestRef = useRef<symbol | null>(null);
  // The padding slider asks for the colour on every step until it lands, and
  // each analysis reads the whole frame, so one is let run at a time; a later
  // ask waits on it, so its undo step holds the colour too.
  const preparingRef = useRef<Promise<void> | null>(null);
  const currentRef = useRef({ output, source });
  currentRef.current = { output, source };

  const analyse = (
    applyBounds: boolean,
    sourceCrop = currentRef.current.output.sourceCrop,
    refresh = false,
  ): Promise<void> => {
    const current = currentRef.current;
    if (!current.source) return Promise.resolve();
    const preparing = !applyBounds && !refresh;
    if (preparing && current.output.recenterInsetColor)
      return Promise.resolve();
    if (preparing && preparingRef.current) return preparingRef.current;
    const request = Symbol();
    requestRef.current = request;
    const read = getRecordingRecenterAnalysis(
      artifactId,
      getPositionMs(),
      sourceCrop,
    )
      .then((analysis) => {
        if (!analysis || requestRef.current !== request) return;
        const latest = currentRef.current;
        if (
          !latest.source ||
          latest.output.sourceCrop.x !== sourceCrop.x ||
          latest.output.sourceCrop.y !== sourceCrop.y ||
          latest.output.sourceCrop.width !== sourceCrop.width ||
          latest.output.sourceCrop.height !== sourceCrop.height
        )
          return;
        const colored = {
          ...latest.output,
          recenterInsetColor: analysis.backgroundColor,
        };
        onOutputChange?.(
          applyBounds && analysis.bounds
            ? recenterScreenshotContent(colored, latest.source, analysis.bounds)
            : colored,
        );
      })
      .catch((error: unknown) => {
        if (requestRef.current === request) requestRef.current = null;
        console.error(
          applyBounds
            ? "Could not detect recording content bounds"
            : "Could not detect recording inset colour",
          error,
        );
      })
      .finally(() => {
        if (preparingRef.current === read) preparingRef.current = null;
      });
    if (preparing) preparingRef.current = read;
    return read;
  };
  const refresh = (sourceCrop: ScreenshotOutputSettings["sourceCrop"]) =>
    analyse(false, sourceCrop, true);
  useRecenterInsetControls("recording", {
    begin: () => {
      pause();
      void analyse(true);
    },
    prepare: () => {
      pause();
      return analyse(false);
    },
    refresh,
  });

  return { refresh };
}
