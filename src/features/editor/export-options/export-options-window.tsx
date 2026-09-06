// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { isTauri } from "@tauri-apps/api/core";
import { UnlistenFn } from "@tauri-apps/api/event";
import { useCallback, useEffect, useRef } from "react";

import { useEditableGeneralSettings } from "../../settings/use-general-settings";
import { selectArtifact, useEditorStore } from "../store";
import { currentExportKind } from "../window-kind";

import {
  hideExportOptions,
  listenToExportOptionsOpened,
  resizeExportOptions,
} from "./api";
import { ExportOptionsWindowView } from "./export-options-window-view";
import {
  DEFAULT_EXPORT_OPTIONS,
  ExportOptionsPatch,
  sendExportOptionsRequest,
  useExportOptionsStore,
} from "./store";
import { useEchoedValue } from "./use-echoed-value";

/**
 * Keeps the window as tall as its form, which changes with the rows a
 * workspace happens to need. Measured from the content rather than assumed,
 * so the configured height is only where the window starts.
 *
 * The window is presented invisible and the fit is what reveals it, so an
 * opening always states a height even when nothing has changed - which is
 * precisely the case from the second opening onwards, where the observer has
 * nothing to report and would otherwise leave the window unseen.
 */
function useContentHeight(contentRef: React.RefObject<HTMLElement | null>) {
  useEffect(() => {
    const element = contentRef.current;
    if (!element || !isTauri()) return;
    let frame = 0;
    let lastHeight = 0;
    const fit = (unchangedToo: boolean) => {
      frame = 0;
      const height = Math.ceil(element.getBoundingClientRect().height);
      if (height === 0 || (height === lastHeight && !unchangedToo)) return;
      lastHeight = height;
      resizeExportOptions(height).catch((cause: unknown) => {
        console.error("Could not fit the export options window", cause);
      });
    };
    // One resize per frame: a row appearing can settle over several layouts.
    const observer = new ResizeObserver(() => {
      frame ||= window.requestAnimationFrame(() => {
        fit(false);
      });
    });
    observer.observe(element);
    // This mount is the first opening; the event carries every later one,
    // which this window stays loaded through.
    fit(true);
    let stopListening: UnlistenFn | undefined;
    let unmounted = false;
    listenToExportOptionsOpened(() => {
      fit(true);
    })
      .then((unlisten) => {
        if (unmounted) unlisten();
        else stopListening = unlisten;
      })
      .catch((cause: unknown) => {
        console.error("Could not follow the export options window", cause);
      });
    return () => {
      unmounted = true;
      stopListening?.();
      observer.disconnect();
      if (frame !== 0) window.cancelAnimationFrame(frame);
    };
  }, [contentRef]);
}

/**
 * The export options as their own window, a non-movable child of the editor
 * it belongs to.
 *
 * Nothing here is owned: the artifact arrives through the shared editor
 * snapshot, the settings through the export options mirror, and every change
 * goes back to the editor as a request so its own coupling rules decide what
 * the setting becomes.
 */
export function ExportOptionsWindow() {
  const kind = currentExportKind() ?? "recording";
  const artifact = useEditorStore(selectArtifact(kind));
  const options =
    useExportOptionsStore((state) => state.snapshots[kind]) ??
    DEFAULT_EXPORT_OPTIONS;
  const contentRef = useRef<HTMLElement>(null);
  useContentHeight(contentRef);
  // Revealing the file is a stored preference rather than a property of this
  // export, so it is read and written where the general settings live.
  const [general, changeGeneral] = useEditableGeneralSettings();
  // The save runs in the editor but is watched from here, so the window has no
  // way out until it ends: closing it would leave nothing showing the progress.
  const isSaving = options.isSaving;

  const close = useCallback(() => {
    hideExportOptions().catch((cause: unknown) => {
      console.error("Could not close the export options", cause);
    });
  }, []);
  const patch = useCallback(
    (values: ExportOptionsPatch) => {
      sendExportOptionsRequest(kind, { type: "patch", values });
    },
    [kind],
  );

  const [fileStem, changeFileStem] = useEchoedValue(
    options.fileStem,
    useCallback(
      (value: string) => {
        patch({ fileStem: value });
      },
      [patch],
    ),
  );

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape" && !isSaving) close();
    };
    window.addEventListener("keydown", onKeyDown);
    return () => {
      window.removeEventListener("keydown", onKeyDown);
    };
  }, [close, isSaving]);

  return (
    <ExportOptionsWindowView
      contentRef={contentRef}
      form={{
        artifact,
        bakeCamera: options.bakeCamera,
        cameraCompression: options.cameraCompression,
        cameraResolutionScalePercent: options.cameraResolutionScalePercent,
        canExport: options.canExport,
        collapseAudio: options.collapseAudio,
        compression: options.compression,
        directory: options.directory,
        enabledAudioTrackCount: options.enabledAudioTrackCount,
        estimatedSizeBytes: options.estimatedSizeBytes,
        extension: options.extension || (artifact?.extension ?? ""),
        fileStem,
        includeCamera: options.includeCamera,
        isEstimatingSize: options.isEstimatingSize,
        isSaving: options.isSaving,
        onBrowse: () => {
          sendExportOptionsRequest(kind, { type: "browse" });
        },
        onCameraCompressionChange: (cameraCompression) => {
          patch({ cameraCompression });
        },
        onCameraResolutionScaleChange: (cameraResolutionScalePercent) => {
          patch({ cameraResolutionScalePercent });
        },
        onCancel: close,
        onCollapseAudioChange: (collapseAudio) => {
          patch({ collapseAudio });
        },
        onCompressionChange: (compression) => {
          patch({ compression });
        },
        onExport: () => {
          // The window stays: it is where the save reports from, and the
          // editor puts it away itself once the file has been written.
          sendExportOptionsRequest(kind, { type: "export" });
        },
        onFileStemChange: changeFileStem,
        onOpenLocationAfterExportChange: (openLocationAfterExport) => {
          changeGeneral({ openLocationAfterExport });
        },
        onResolutionScaleChange: (resolutionScalePercent) => {
          patch({ resolutionScalePercent });
        },
        openLocationAfterExport: general?.openLocationAfterExport ?? false,
        resolutionScalePercent: options.resolutionScalePercent,
      }}
      isSaving={isSaving}
      onClose={close}
      progress={{
        // A screenshot is written in one pass the backend cannot interrupt.
        cancellable: kind === "recording",
        etaSeconds: options.etaSeconds,
        isAudioOnly: options.isAudioOnly,
        isCancelingSave: options.isCancelingSave,
        isRecording: kind === "recording",
        onCancel: () => {
          sendExportOptionsRequest(kind, { type: "cancel" });
        },
        phase: options.savePhase,
        progress: options.saveProgress,
      }}
    />
  );
}
