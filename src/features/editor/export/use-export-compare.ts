// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later
/* eslint-disable @eslint-react/set-state-in-effect -- The compared pictures are made outside React, as the estimate is. */

import { convertFileSrc } from "@tauri-apps/api/core";
import { useEffect, useRef, useState } from "react";

import { usePreviewStill } from "../recording/preview/preview-still-store";

import {
  previewRecordingExportFrame,
  previewScreenshotExport,
} from "./export-preview-api";
import {
  RecordingExportInputs,
  recordingProcessingOptions,
} from "./recording-export-inputs";

import type { ExportPreviewFrame } from "../../../bindings/ExportPreviewFrame";
import type { ScreenshotFormat } from "../../../bindings/ScreenshotFormat";
import type { ScreenshotWorkspaceOutputSettings } from "../screenshot/screenshot-output";
import type {
  ToolPanelComparedFrame,
  ToolPanelExportCompare,
} from "../tools/tool-panel-export";

/** Long enough that stepping through the looks, or a playhead dragged,
 * asks for the pictures once, where it stops. */
const COMPARE_DEBOUNCE_MS = 350;

type Made = { frame: ToolPanelComparedFrame; key: string };
type Pictures = {
  exported: Made | null;
  failed: boolean;
  original: Made | null;
  seeThrough: boolean;
};

const NOTHING: Pictures = {
  exported: null,
  failed: false,
  original: null,
  seeThrough: false,
};

const shown = (frame: ExportPreviewFrame): ToolPanelComparedFrame => ({
  canvas: frame.canvas,
  height: frame.height,
  src: convertFileSrc(frame.path),
  width: frame.width,
});

/**
 * What the Export island compares while it is out: a recording's frame at
 * the playhead as drawn, never encoded, beside the same frame at the chosen
 * look; or a screenshot as drawn beside its JPEG or WebP. A PNG screenshot
 * is written as drawn, so it has nothing to compare. The Original frame
 * depends on everything the export draws but its look, so choosing another
 * look makes only the exported one again. Null where there is nothing to
 * compare.
 */
export function useExportCompare(
  inputs: RecordingExportInputs,
  {
    isOpen,
    screenshot,
  }: {
    isOpen: boolean;
    screenshot: {
      format: ScreenshotFormat;
      output: ScreenshotWorkspaceOutputSettings;
    };
  },
): ToolPanelExportCompare | null {
  const positionMs = Math.round(usePreviewStill((still) => still.positionMs));
  const { artifact, compression, resolutionScalePercent } = inputs;
  const recording =
    artifact?.kind === "recording" &&
    artifact.canCompress &&
    inputs.includePrimaryVideo &&
    artifact.width > 0 &&
    artifact.height > 0
      ? artifact
      : null;
  const isLossy =
    artifact?.kind === "screenshot" && screenshot.format !== "png";
  // The screenshot's two pictures are made together, so they share a key.
  const originalKey = recording
    ? JSON.stringify([
        recording.id,
        positionMs,
        inputs.bakeCamera,
        inputs.includeCamera,
        inputs.cameraOverlay,
        inputs.cursorEffects,
        inputs.keyboardEffects,
        inputs.recordingOutput,
        inputs.recordingTimelineEdit,
      ])
    : isLossy
      ? JSON.stringify([artifact.id, screenshot.format, screenshot.output])
      : null;
  const exportKey =
    originalKey === null || !recording
      ? originalKey
      : `${originalKey}:${compression.toString()}:${resolutionScalePercent.toString()}`;

  const [made, setMade] = useState<Pictures>(NOTHING);
  const madeRef = useRef(made);
  madeRef.current = made;
  const latestRef = useRef({ inputs, positionMs, screenshot });
  latestRef.current = { inputs, positionMs, screenshot };
  // Raised by every request and every cleanup, so an answer arriving for
  // anything but the latest request is dropped.
  const generationRef = useRef(0);

  const artifactId = artifact?.id;
  useEffect(() => {
    setMade(NOTHING);
  }, [artifactId]);

  useEffect(() => {
    if (!isOpen || originalKey === null || exportKey === null) return;
    const current = madeRef.current;
    if (
      current.original?.key === originalKey &&
      current.exported?.key === exportKey
    )
      return;
    setMade((state) => ({ ...state, failed: false }));
    generationRef.current += 1;
    const generation = generationRef.current;
    const isCurrent = () => generationRef.current === generation;
    const failed = (cause: unknown) => {
      if (!isCurrent()) return;
      console.error("Could not make the compared export pictures", cause);
      setMade((state) => ({ ...state, failed: true }));
    };
    // With nothing on screen there is no choice to wait out.
    const delay = current.original === null ? 0 : COMPARE_DEBOUNCE_MS;
    const timer = window.setTimeout(() => {
      const latest = latestRef.current;
      const artifactId = latest.inputs.artifact?.id;
      if (artifactId === undefined) return;
      if (latest.inputs.artifact?.kind === "screenshot") {
        void previewScreenshotExport(
          latest.screenshot.format,
          latest.screenshot.output,
        )
          .then((pair) => {
            if (!isCurrent() || pair === null) return;
            setMade({
              exported: { frame: shown(pair.exported), key: exportKey },
              failed: false,
              original: { frame: shown(pair.original), key: originalKey },
              seeThrough: pair.seeThrough,
            });
          })
          .catch(failed);
        return;
      }
      const make = (side: "original" | "export") =>
        previewRecordingExportFrame({
          artifactId,
          options: recordingProcessingOptions(latest.inputs),
          positionMs: latest.positionMs,
          side,
        });
      // Both sides are made at once; each answer is null where a newer
      // frame of its side was asked for, which takes over.
      const original =
        madeRef.current.original?.key === originalKey
          ? null
          : make("original").then((frame) => {
              if (!isCurrent() || frame === null) return;
              setMade((state) => ({
                ...state,
                original: { frame: shown(frame), key: originalKey },
              }));
            });
      const exported = make("export").then((frame) => {
        if (!isCurrent() || frame === null) return;
        setMade((state) => ({
          ...state,
          exported: { frame: shown(frame), key: exportKey },
        }));
      });
      void Promise.all([original, exported]).catch(failed);
    }, delay);
    return () => {
      generationRef.current += 1;
      window.clearTimeout(timer);
    };
  }, [exportKey, isOpen, originalKey]);

  if (originalKey === null) return null;
  const isUpdating =
    !made.failed &&
    (made.original?.key !== originalKey || made.exported?.key !== exportKey);
  // An exported frame of another moment than the Original one held would
  // set two different pictures side by side, so it waits for its pair.
  const exported =
    made.original &&
    made.exported &&
    (made.exported.key === made.original.key ||
      made.exported.key.startsWith(`${made.original.key}:`))
      ? made.exported.frame
      : null;
  return {
    exported,
    failed: made.failed,
    isUpdating,
    original: made.original?.frame ?? null,
    seeThrough: made.seeThrough,
  };
}
