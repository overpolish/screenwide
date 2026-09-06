// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef } from "react";

import { EditorKind } from "../types";

import {
  ExportOptionsPatch,
  ExportOptionsSnapshot,
  useExportOptionsRequestStore,
  useExportOptionsStore,
} from "./store";

/**
 * What the options window can ask for. Each one is an editor handler that
 * already exists, so the coupling rules between compression and output size
 * run exactly as they do for a control inside the editor.
 */
export type ExportOptionsHandlers = {
  onBrowse?: () => void;
  onCameraCompressionChange?: (compression: number) => void;
  onCameraResolutionScaleChange?: (scale: number) => void;
  onCancelSave?: () => void;
  onCollapseAudioChange?: (collapse: boolean) => void;
  onCompressionChange?: (compression: number) => void;
  onExport?: () => void;
  onFileStemChange?: (fileStem: string) => void;
  onResolutionScaleChange?: (scale: number) => void;
};

/**
 * The last request each workspace has acted on. Module state rather than a
 * ref: the request outlives any one mount of the editor panel, and a remount
 * must not replay an export that has already begun.
 */
const appliedSeq: Record<EditorKind, number> = { recording: 0, screenshot: 0 };

const applyPatch = (values: ExportOptionsPatch, on: ExportOptionsHandlers) => {
  if (values.fileStem !== undefined) on.onFileStemChange?.(values.fileStem);
  if (values.compression !== undefined) {
    on.onCompressionChange?.(values.compression);
  }
  if (values.cameraCompression !== undefined) {
    on.onCameraCompressionChange?.(values.cameraCompression);
  }
  if (values.resolutionScalePercent !== undefined) {
    on.onResolutionScaleChange?.(values.resolutionScalePercent);
  }
  if (values.cameraResolutionScalePercent !== undefined) {
    on.onCameraResolutionScaleChange?.(values.cameraResolutionScalePercent);
  }
  if (values.collapseAudio !== undefined) {
    on.onCollapseAudioChange?.(values.collapseAudio);
  }
};

/**
 * The editor's half of the export options window: it publishes the settings
 * the form shows, and applies what the form asks for.
 *
 * Only the editor window owns this state, so the options window never writes
 * a setting itself; it sends a request, and reads the result back out of the
 * next published snapshot.
 */
export function useExportOptionsBridge(
  kind: EditorKind,
  snapshot: ExportOptionsSnapshot,
  handlers: ExportOptionsHandlers,
) {
  const lastRequest = useExportOptionsRequestStore(
    (state) => state.lastRequest,
  );
  const handlersRef = useRef(handlers);

  useEffect(() => {
    handlersRef.current = handlers;
  });

  useEffect(() => {
    // A no-op when nothing changed, so republishing on every render costs a
    // shallow comparison rather than a `localStorage` write.
    useExportOptionsStore.getState().publish(kind, snapshot);
  }, [kind, snapshot]);

  useEffect(() => {
    if (
      !lastRequest ||
      lastRequest.kind !== kind ||
      lastRequest.seq <= appliedSeq[kind]
    ) {
      return;
    }
    appliedSeq[kind] = lastRequest.seq;
    const { request } = lastRequest;
    if (request.type === "browse") handlersRef.current.onBrowse?.();
    else if (request.type === "cancel") handlersRef.current.onCancelSave?.();
    else if (request.type === "export") handlersRef.current.onExport?.();
    else applyPatch(request.values, handlersRef.current);
  }, [kind, lastRequest]);
}
