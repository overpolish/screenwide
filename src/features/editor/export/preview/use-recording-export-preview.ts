// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later
/* eslint-disable @eslint-react/set-state-in-effect -- The preview frames are made outside React, as the estimate is. */

import { useEffect, useRef, useState } from "react";

import { usePreviewStill } from "../../recording/preview/preview-still-store";
import { resolutionScales } from "../../resolution";
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
  // At Original size and compression the export is the Original frame.
  const isOriginal =
    recording !== null &&
    compression === 0 &&
    resolutionScalePercent === resolutionScales(recording)[0];

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

  const artifactId = artifact?.id;
  useEffect(() => {
    setMade({ exported: null, failed: false, original: null });
  }, [artifactId]);

  useEffect(() => {
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
      void (async () => {
        try {
          const held = madeRef.current.original;
          const original =
            held?.key === originalKey ? held.frame : await make("original");
          // Null where a newer frame was asked for, which takes over.
          if (generationRef.current !== generation || original === null) {
            return;
          }
          setMade((state) => ({
            ...state,
            original: { frame: original, key: originalKey },
          }));
          const exported = isOriginal ? original : await make("export");
          if (generationRef.current !== generation || exported === null) {
            return;
          }
          setMade((state) => ({
            ...state,
            exported: { frame: exported, key: exportKey },
          }));
        } catch (cause) {
          if (generationRef.current !== generation) return;
          console.error("Could not make the export preview", cause);
          setMade((state) => ({ ...state, failed: true }));
        }
      })();
    }, PREVIEW_DEBOUNCE_MS);
    return () => {
      generationRef.current += 1;
      window.clearTimeout(timer);
    };
  }, [exportKey, isExportOpen, isOriginal, originalKey, positionMs]);
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
