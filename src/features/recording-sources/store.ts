// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { create } from "zustand";
import { createJSONStorage, persist } from "zustand/middleware";

import { MonitorDetails, RecordingMode, Region, WindowDetails } from "./types";

const STORE_NAME = "screenwide-recording-source";

type RecordingSourceStore = {
  /**
   * A shortcut-initiated screenshot borrows the region overlay, whatever the
   * recording mode happens to be, and hands the region straight to a capture.
   */
  isScreenshotCapture: boolean;
  recordingMode: RecordingMode;
  region: Region;
  regionAspectRatio: number | undefined;
  selectedMonitor: MonitorDetails | null;
  selectedWindow: WindowDetails | null;
  setRecordingMode: (mode: RecordingMode) => void;
  setRegion: (region: Region) => void;
  setRegionAspectRatio: (ratio: number | undefined) => void;
  setScreenshotCapture: (capturing: boolean) => void;
  setSelectedMonitor: (monitor: MonitorDetails) => void;
  setSelectedWindow: (window: WindowDetails | null) => void;
};

export const useRecordingSourceStore = create<RecordingSourceStore>()(
  persist(
    (set) => ({
      isScreenshotCapture: false,
      recordingMode: "screen",
      region: {
        position: { x: 160, y: 90 },
        size: { height: 720, width: 1280 },
      },
      regionAspectRatio: undefined,
      selectedMonitor: null,
      selectedWindow: null,
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
      name: STORE_NAME,
      storage: createJSONStorage(() => localStorage),
    },
  ),
);

export const synchronizeRecordingSourceStore = (event: StorageEvent) => {
  if (event.key === STORE_NAME) {
    void useRecordingSourceStore.persist.rehydrate();
  }
};
