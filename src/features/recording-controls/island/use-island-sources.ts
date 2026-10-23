// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useState } from "react";

import {
  listMonitors,
  selectedWindowStatus,
} from "../../recording-sources/api";
import {
  findCurrentMonitor,
  orderMonitorsForNavigation,
} from "../../recording-sources/monitor-selection";
import {
  MONITOR_THUMBNAIL_INTERVAL_MS,
  refreshMonitorThumbnails,
} from "../../recording-sources/monitor-thumbnails";
import { useRecordingSourceStore } from "../../recording-sources/store";
import { type MonitorDetails } from "../../recording-sources/types";

const SOURCE_CHECK_INTERVAL_MS = 1_500;

/**
 * Clears a window that can no longer be recorded while window mode is active.
 * `refreshIcon` repoints the remembered app icon at a freshly extracted file:
 * it lives in the temp directory, which macOS purges behind the stored path.
 * The write always lands, even for an unchanged path, so an icon that already
 * failed to load retries against the recreated file.
 */
const checkSelectedWindow = async (
  isActive: () => boolean,
  refreshIcon: boolean,
) => {
  const selected = useRecordingSourceStore.getState().selectedWindow;
  if (!selected) return;
  const { appIconPath, available } = await selectedWindowStatus(selected);
  if (!isActive()) return;
  const { recordingMode, selectedWindow, setSelectedWindow } =
    useRecordingSourceStore.getState();
  if (
    selectedWindow?.id !== selected.id ||
    selectedWindow.pid !== selected.pid
  ) {
    return;
  }
  if (!available && recordingMode === "window") {
    setSelectedWindow(null);
  } else if (refreshIcon) {
    setSelectedWindow({ ...selectedWindow, appIconPath });
  }
};

/** The displays in their arrangement, with the chosen one kept current:
 * still chosen when rearranged, the primary display when it is gone. */
const readDisplays = async (): Promise<MonitorDetails[]> => {
  const available = await listMonitors();
  const { selectedMonitor, setSelectedMonitor } =
    useRecordingSourceStore.getState();
  const current = findCurrentMonitor(available, selectedMonitor);
  if (current && JSON.stringify(current) !== JSON.stringify(selectedMonitor)) {
    setSelectedMonitor(current);
  }
  return orderMonitorsForNavigation(available, "horizontal");
};

/**
 * What the controls record from, kept current while they are open and idle:
 * the displays, read on a short cadence so one plugged in or taken away is
 * noticed; stills of them, retaken on the chooser's cadence while a display
 * is what is set up; and the chosen window, checked as the displays are,
 * though outside window mode only its icon, once, as the controls open.
 */
export function useIslandSources(isActive: boolean) {
  const [displays, setDisplays] = useState<MonitorDetails[]>([]);
  const recordingMode = useRecordingSourceStore((state) => state.recordingMode);

  useEffect(() => {
    if (!isActive) return;
    void refreshMonitorThumbnails();
    if (recordingMode !== "screen") return;
    const interval = window.setInterval(() => {
      void refreshMonitorThumbnails();
    }, MONITOR_THUMBNAIL_INTERVAL_MS);
    return () => {
      window.clearInterval(interval);
    };
  }, [isActive, recordingMode]);

  useEffect(() => {
    if (!isActive) return;
    let disposed = false;
    let checking = false;
    const check = async (isFirst: boolean) => {
      if (checking) return;
      checking = true;
      try {
        const next = await readDisplays();
        if (disposed) return;
        setDisplays(next);
        const mode = useRecordingSourceStore.getState().recordingMode;
        if (mode === "window" || isFirst) {
          await checkSelectedWindow(() => !disposed, isFirst);
        }
      } catch (error) {
        console.error("Could not check the recording sources", error);
      } finally {
        checking = false;
      }
    };
    void check(true);
    const interval = window.setInterval(() => {
      if (!disposed) void check(false);
    }, SOURCE_CHECK_INTERVAL_MS);
    return () => {
      disposed = true;
      window.clearInterval(interval);
    };
  }, [isActive]);

  return displays;
}
