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
 * The recording's content analysis, published for the crop panel's commit and
 * the Select panel's padding controls. Both read the frame under the
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
  const currentRef = useRef({ output, source });
  currentRef.current = { output, source };

  const analyse = (
    applyBounds: boolean,
    sourceCrop = currentRef.current.output.sourceCrop,
    refresh = false,
  ) => {
    const current = currentRef.current;
    if (!current.source) return;
    if (!applyBounds && !refresh && current.output.recenterInsetColor) return;
    const request = Symbol();
    requestRef.current = request;
    void getRecordingRecenterAnalysis(artifactId, getPositionMs(), sourceCrop)
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
      });
  };
  const refresh = (sourceCrop: ScreenshotOutputSettings["sourceCrop"]) => {
    analyse(false, sourceCrop, true);
  };
  useRecenterInsetControls("recording", {
    begin: () => {
      pause();
      analyse(true);
    },
    prepare: () => {
      pause();
      analyse(false);
    },
    refresh,
  });

  return { refresh };
}
