// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useRef } from "react";

import { ScreenshotOutputSettings } from "../screenshot-output";
import {
  getRecordingRecenterAnalysis,
  recenterScreenshotContent,
  resetScreenshotRecenter,
} from "../screenshot-recenter";

export function useRecordingRecenter({
  artifactId,
  getPositionMs,
  onOutputChange,
  output,
  source,
}: {
  artifactId: number;
  getPositionMs: () => number;
  output: ScreenshotOutputSettings;
  onOutputChange?: (settings: ScreenshotOutputSettings) => void;
  source?: { height: number; width: number };
}) {
  const requestRef = useRef<symbol | null>(null);
  const currentRef = useRef({ output, source });
  currentRef.current = { output, source };

  const analyse = (applyBounds: boolean) => {
    const current = currentRef.current;
    if (!current.source) return;
    if (!applyBounds && current.output.recenterInsetColor) return;
    const sourceCrop = current.output.sourceCrop;
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
  const begin = () => {
    analyse(true);
  };
  const prepare = () => {
    analyse(false);
  };

  const reset = () => {
    const current = currentRef.current;
    if (!current.source) return;
    requestRef.current = null;
    onOutputChange?.(resetScreenshotRecenter(current.output, current.source));
  };

  return { begin, prepare, reset };
}
