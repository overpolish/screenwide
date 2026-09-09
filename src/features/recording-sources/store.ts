// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { convertFileSrc } from "@tauri-apps/api/core";
import { create } from "zustand";
import { createJSONStorage, persist } from "zustand/middleware";

import {
  MonitorDetails,
  MonitorThumbnail,
  RecordingMode,
  Region,
  WindowDetails,
} from "./types";

const STORE_NAME = "screenwide-recording-source";

type RecordingSourceStore = {
  /**
   * A shortcut-initiated screenshot borrows the region overlay, whatever the
   * recording mode happens to be, and hands the region straight to a capture.
   */
  isScreenshotCapture: boolean;
  /** A still of each attached display by its id, as an asset URL. Cache
   * files behind live hardware, so they are never persisted. */
  monitorThumbnails: Record<number, string>;
  recordingMode: RecordingMode;
  region: Region;
  regionAspectRatio: number | undefined;
  selectedMonitor: MonitorDetails | null;
  selectedWindow: WindowDetails | null;
  setMonitorThumbnails: (thumbnails: MonitorThumbnail[]) => void;
  setRecordingMode: (mode: RecordingMode) => void;
  setRegion: (region: Region) => void;
  setRegionAspectRatio: (ratio: number | undefined) => void;
  setScreenshotCapture: (capturing: boolean) => void;
  setSelectedMonitor: (monitor: MonitorDetails) => void;
  setSelectedWindow: (window: WindowDetails | null) => void;
};

export const mergeRecordingSourceState = <
  State extends { regionAspectRatio: number | undefined },
>(
  persistedState: unknown,
  currentState: State,
): State => {
  const persisted = persistedState as Partial<State> | undefined;

  return {
    ...currentState,
    ...persisted,
    // JSON omits `undefined`, which is how the linked control represents an
    // unlinked ratio. Treat an absent persisted key as that explicit state so
    // another window cannot retain the ratio from its previous store snapshot.
    regionAspectRatio: persisted?.regionAspectRatio,
  };
};

export const useRecordingSourceStore = create<RecordingSourceStore>()(
  persist(
    (set) => ({
      isScreenshotCapture: false,
      monitorThumbnails: {},
      recordingMode: "screen",
      region: {
        position: { x: 160, y: 90 },
        size: { height: 720, width: 1280 },
      },
      regionAspectRatio: undefined,
      selectedMonitor: null,
      selectedWindow: null,
      setMonitorThumbnails: (thumbnails) => {
        set({
          monitorThumbnails: Object.fromEntries(
            thumbnails.map(({ id, path }) => [
              id,
              // The file keeps its name across refreshes, so the webview would
              // otherwise keep serving the still it already cached.
              `${convertFileSrc(path)}?v=${String(Date.now())}`,
            ]),
          ),
        });
      },
      setRecordingMode: (recordingMode) => {
        set({ recordingMode });
      },
      setRegion: (region) => {
        set({ region });
      },
      setRegionAspectRatio: (regionAspectRatio) => {
        set({ regionAspectRatio });
      },
      setScreenshotCapture: (isScreenshotCapture) => {
        set({ isScreenshotCapture });
      },
      setSelectedMonitor: (selectedMonitor) => {
        set({ selectedMonitor });
      },
      setSelectedWindow: (selectedWindow) => {
        set({ selectedWindow });
      },
    }),
    {
      merge: mergeRecordingSourceState,
      name: STORE_NAME,
      // The display stills are captured fresh on every run, so they never
      // join the persisted snapshot.
      partialize: ({ monitorThumbnails: _monitorThumbnails, ...state }) =>
        state,
      storage: createJSONStorage(() => localStorage),
    },
  ),
);

export const synchronizeRecordingSourceStore = (event: StorageEvent) => {
  if (event.key === STORE_NAME) {
    void useRecordingSourceStore.persist.rehydrate();
  }
};
