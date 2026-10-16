// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later
/* eslint-disable @eslint-react/set-state-in-effect -- The preview frames are made outside React, as the estimate is. */

import { useEffect, useRef, useState } from "react";

import { usePreviewStill } from "../../recording/preview/preview-still-store";
import { useExportOptionsOpen } from "../options-window/use-export-options-open";
import {
  RecordingExportInputs,
  recordingProcessingOptions,
} from "../recording-export-inputs";

import { previewRecordingExportFrame } from "./export-preview-api";
import { useExportPreviewStore } from "./export-preview-store";

import type { ExportPreviewFrame } from "../../../../bindings/ExportPreviewFrame";

/** Long enough that stepping through the size or compression choices asks
 * for the frame once, where the user stops. */
const PREVIEW_DEBOUNCE_MS = 350;

type Made = { frame: ExportPreviewFrame; key: string };

/**
 * Makes the frame under the playhead at Original and as the export will
 * write it, while the export window is open, and publishes both for that
 * window. The Original frame depends on everything the export draws but its
 * size and compression, so changing those makes only the exported one again.
 */
export function useRecordingExportPreview(inputs: RecordingExportInputs) {
  const { isExportOpen } = useExportOptionsOpen("recording");
  const positionMs = Math.round(usePreviewStill((still) => still.positionMs));
  const { artifact, compression, resolutionScalePercent } = inputs;
  const recording =
    artifact?.kind === "recording" &&
    inputs.includePrimaryVideo &&
    artifact.width > 0 &&
    artifact.height > 0
      ? artifact
      : null;
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
    : null;
  const exportKey =
    originalKey === null
      ? null
      : `${originalKey}:${compression.toString()}:${resolutionScalePercent.toString()}`;

  const [made, setMade] = useState<{
    exported: Made | null;
    failed: boolean;
    original: Made | null;
  }>({ exported: null, failed: false, original: null });
  const madeRef = useRef(made);
  madeRef.current = made;
  const inputsRef = useRef(inputs);
  inputsRef.current = inputs;
  // Raised by every request and every cleanup, so an answer arriving for
  // anything but the latest request is dropped.
  const generationRef = useRef(0);
  const wasOpenRef = useRef(false);

  const artifactId = artifact?.id;
  useEffect(() => {
    setMade({ exported: null, failed: false, original: null });
  }, [artifactId]);

  useEffect(() => {
    const justOpened = isExportOpen && !wasOpenRef.current;
    wasOpenRef.current = isExportOpen;
    if (!isExportOpen || originalKey === null || exportKey === null) return;
    const current = madeRef.current;
    if (
      current.original?.key === originalKey &&
      current.exported?.key === exportKey
    ) {
      return;
    }
    setMade((state) => ({ ...state, failed: false }));
    generationRef.current += 1;
    const generation = generationRef.current;
    const isCurrent = () => generationRef.current === generation;
    // With nothing on screen there is no choice to wait out.
    const delay =
      justOpened || current.original === null ? 0 : PREVIEW_DEBOUNCE_MS;
    const timer = window.setTimeout(() => {
      const latest = inputsRef.current;
      const artifactId = latest.artifact?.id;
      if (artifactId === undefined) return;
      const make = (side: "original" | "export") =>
        previewRecordingExportFrame({
          artifactId,
          options: recordingProcessingOptions(latest),
          positionMs,
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
                original: { frame, key: originalKey },
              }));
            });
      const exported = make("export").then((frame) => {
        if (!isCurrent() || frame === null) return;
        setMade((state) => ({
          ...state,
          exported: { frame, key: exportKey },
        }));
      });
      void Promise.all([original, exported]).catch((cause: unknown) => {
        if (!isCurrent()) return;
        console.error("Could not make the export preview", cause);
        setMade((state) => ({ ...state, failed: true }));
      });
    }, delay);
    return () => {
      generationRef.current += 1;
      window.clearTimeout(timer);
    };
  }, [exportKey, isExportOpen, originalKey, positionMs]);
  const isUpdating =
    isExportOpen &&
    originalKey !== null &&
    !made.failed &&
    (made.original?.key !== originalKey || made.exported?.key !== exportKey);
  // An exported frame of another moment than the Original one held would
  // set two different pictures side by side, so it waits for its pair.
  const exported =
    made.original && made.exported?.key.startsWith(`${made.original.key}:`)
      ? made.exported.frame
      : null;
  useEffect(() => {
    useExportPreviewStore.getState().publish("recording", {
      available: originalKey !== null,
      exported,
      failed: made.failed,
      isUpdating,
      original: made.original?.frame ?? null,
    });
  }, [exported, isUpdating, made, originalKey]);
}
