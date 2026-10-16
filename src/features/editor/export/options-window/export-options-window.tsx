// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { convertFileSrc, isTauri } from "@tauri-apps/api/core";
import { UnlistenFn } from "@tauri-apps/api/event";
import { useCallback, useEffect, useRef } from "react";

import { useEditableGeneralSettings } from "../../../settings/use-general-settings";
import { selectArtifact, useEditorStore } from "../../store";
import { currentExportKind } from "../../window-kind";
import { ExportPreviewCompare } from "../preview/export-preview-compare";
import { useExportPreviewStore } from "../preview/export-preview-store";

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

/** The window's width alone, and with the recording's preview beside the
 * form: the form keeps the column it has alone. */
const WINDOW_WIDTH = 480;
const WINDOW_WIDTH_WITH_PREVIEW = 920;

/**
 * Keeps the window as tall as its form, which changes with the rows a
 * workspace happens to need, and `width` wide. Measured from the content
 * rather than assumed, so the configured height is only where the window
 * starts.
 *
 * The window is presented invisible and the fit is what reveals it, so an
 * opening always states a size even when nothing has changed - which is
 * precisely the case from the second opening onwards, where the observer has
 * nothing to report and would otherwise leave the window unseen.
 */
function useContentSize(
  contentRef: React.RefObject<HTMLElement | null>,
  width: number,
) {
  const widthRef = useRef(width);
  const fitRef = useRef<((unchangedToo: boolean) => void) | null>(null);
  useEffect(() => {
    const element = contentRef.current;
    if (!element || !isTauri()) return;
    let frame = 0;
    let last = { height: 0, width: 0 };
    const fit = (unchangedToo: boolean) => {
      frame = 0;
      const size = {
        height: Math.ceil(element.getBoundingClientRect().height),
        width: widthRef.current,
      };
      const unchanged =
        size.height === last.height && size.width === last.width;
      if (size.height === 0 || (unchanged && !unchangedToo)) return;
      last = size;
      resizeExportOptions(size.width, size.height).catch((cause: unknown) => {
        console.error("Could not fit the export options window", cause);
      });
    };
    fitRef.current = fit;
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
      fitRef.current = null;
      stopListening?.();
      observer.disconnect();
      if (frame !== 0) window.cancelAnimationFrame(frame);
    };
  }, [contentRef]);
  useEffect(() => {
    widthRef.current = width;
    fitRef.current?.(false);
  }, [width]);
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
  const recordingPreview = useExportPreviewStore(
    (state) => state.snapshots.recording,
  );
  const preview =
    kind === "recording" && recordingPreview?.available
      ? recordingPreview
      : null;
  useContentSize(
    contentRef,
    preview ? WINDOW_WIDTH_WITH_PREVIEW : WINDOW_WIDTH,
  );
  const frame = (
    shown: { height: number; path: string; width: number } | null,
  ) => shown && { ...shown, src: convertFileSrc(shown.path) };
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
        cameraCompression: options.cameraCompression,
        cameraOutput: options.cameraOutput,
        cameraResolutionScalePercent: options.cameraResolutionScalePercent,
        canExport: options.canExport,
        collapseAudio: options.collapseAudio,
        compression: options.compression,
        deleteProjectAfterExport: options.deleteProjectAfterExport,
        directory: options.directory,
        enabledAudioTrackCount: options.enabledAudioTrackCount,
        estimatedSizeBytes: options.estimatedSizeBytes,
        extension: options.extension || (artifact?.extension ?? ""),
        fileStem,
        hasTranscript: options.hasTranscript,
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
        onDeleteProjectAfterExportChange: (deleteProjectAfterExport) => {
          patch({ deleteProjectAfterExport });
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
        onSubtitlesChange: (subtitles) => {
          patch({ subtitles });
        },
        openLocationAfterExport: general?.openLocationAfterExport ?? false,
        recordingOutput: options.recordingOutput,
        resolutionScalePercent: options.resolutionScalePercent,
        subtitles: options.subtitles,
      }}
      isSaving={isSaving}
      onClose={close}
      preview={
        preview ? (
          <ExportPreviewCompare
            className="flex-1"
            exported={frame(preview.exported)}
            failed={preview.failed}
            isUpdating={preview.isUpdating}
            original={frame(preview.original)}
          />
        ) : undefined
      }
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
