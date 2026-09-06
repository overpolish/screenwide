// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { create } from "zustand";
import { createJSONStorage, persist } from "zustand/middleware";

import { EditorKind } from "../types";
import { ExportPhase } from "../use-export-progress";

/**
 * Everything the export form needs that the editor window owns.
 *
 * The artifact is deliberately not here: it already reaches every window
 * through the editor snapshot store, and duplicating it would mean mirroring
 * the whole capture across `localStorage` on every keystroke.
 */
export type ExportOptionsSnapshot = {
  bakeCamera: boolean;
  cameraCompression: number;
  cameraResolutionScalePercent: number;
  canExport: boolean;
  collapseAudio: boolean;
  compression: number;
  directory: string | null;
  enabledAudioTrackCount: number;
  estimatedSizeBytes: number | null;
  /** How long the running save still has, once that can be measured. */
  etaSeconds: number | null;
  /** What saving delivers, which is not always the artifact's extension. */
  extension: string;
  fileStem: string;
  includeCamera: boolean;
  /** No video is being written, so the save is described as audio. */
  isAudioOnly: boolean;
  isCancelingSave: boolean;
  isEstimatingSize: boolean;
  isSaving: boolean;
  resolutionScalePercent: number;
  savePhase: ExportPhase;
  /** Percent complete, or `null` while the save cannot report a figure. */
  saveProgress: number | null;
};

/** The settings the options window may ask the editor to change. */
export type ExportOptionsPatch = Partial<
  Pick<
    ExportOptionsSnapshot,
    | "cameraCompression"
    | "cameraResolutionScalePercent"
    | "collapseAudio"
    | "compression"
    | "fileStem"
    | "resolutionScalePercent"
  >
>;

export type ExportOptionsRequest =
  | { type: "browse" }
  | { type: "cancel" }
  | { type: "export" }
  | { type: "patch"; values: ExportOptionsPatch };

export type ExportOptionsMessage = {
  kind: EditorKind;
  request: ExportOptionsRequest;
  /** Rising, so the editor can tell a repeat delivery from a new request. */
  seq: number;
};

export const DEFAULT_EXPORT_OPTIONS: ExportOptionsSnapshot = {
  bakeCamera: false,
  cameraCompression: 0,
  cameraResolutionScalePercent: 100,
  canExport: false,
  collapseAudio: false,
  compression: 0,
  directory: null,
  enabledAudioTrackCount: 0,
  estimatedSizeBytes: null,
  etaSeconds: null,
  extension: "",
  fileStem: "",
  includeCamera: false,
  isAudioOnly: false,
  isCancelingSave: false,
  isEstimatingSize: false,
  isSaving: false,
  resolutionScalePercent: 100,
  savePhase: "recording",
  saveProgress: null,
};

type ExportOptionsStore = {
  publish: (kind: EditorKind, snapshot: ExportOptionsSnapshot) => void;
  snapshots: Partial<Record<EditorKind, ExportOptionsSnapshot>>;
};

const STORE_NAME = "screenwide-export-options";
const REQUEST_STORE_NAME = `${STORE_NAME}-request`;

// Wall-clock seeded so a reloaded options window cannot restart below the
// sequence the editor has already applied.
let lastSeq = 0;
const nextSeq = () => {
  lastSeq = Math.max(lastSeq + 1, Date.now());
  return lastSeq;
};

const isSameSnapshot = (
  a: ExportOptionsSnapshot | undefined,
  b: ExportOptionsSnapshot,
) =>
  a !== undefined &&
  (Object.keys(b) as (keyof ExportOptionsSnapshot)[]).every(
    (key) => a[key] === b[key],
  );

export const useExportOptionsStore = create<ExportOptionsStore>()(
  persist(
    (set) => ({
      publish: (kind, snapshot) => {
        set((state) =>
          isSameSnapshot(state.snapshots[kind], snapshot)
            ? state
            : { snapshots: { ...state.snapshots, [kind]: snapshot } },
        );
      },
      snapshots: {},
    }),
    {
      name: STORE_NAME,
      partialize: (state) => ({ snapshots: state.snapshots }),
      storage: createJSONStorage(() => localStorage),
    },
  ),
);

/**
 * The latest ask from an options window, held apart from the settings mirror
 * so sending one never writes the mirror back: only the editor publishes
 * settings, and only the options window asks for changes.
 */
export const useExportOptionsRequestStore = create<{
  lastRequest: ExportOptionsMessage | null;
}>()(() => ({ lastRequest: null }));

/** Asks the editor that owns `kind` to make a change. */
export const sendExportOptionsRequest = (
  kind: EditorKind,
  request: ExportOptionsRequest,
) => {
  const message: ExportOptionsMessage = { kind, request, seq: nextSeq() };
  localStorage.setItem(REQUEST_STORE_NAME, JSON.stringify(message));
};

/** Carries the editor's settings out, and the options window's asks back. */
export const synchronizeExportOptionsStore = (event: StorageEvent) => {
  if (event.key === STORE_NAME) {
    void useExportOptionsStore.persist.rehydrate();
  } else if (event.key === REQUEST_STORE_NAME && event.newValue) {
    try {
      useExportOptionsRequestStore.setState({
        lastRequest: JSON.parse(event.newValue) as ExportOptionsMessage,
      });
    } catch {
      // Ignore malformed cross-window messages.
    }
  }
};
