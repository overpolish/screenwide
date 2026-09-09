// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { useCallback, useEffect, useRef, useState } from "react";

import { CircularProgress } from "../../components/base/circular-progress/circular-progress";

import {
  collapseRecordingSourceSelector,
  getRecordingSourceSelectorState,
  listMonitors,
  listWindows,
} from "./api";
import { findCurrentMonitor } from "./monitor-selection";
import { MonitorSelector } from "./monitor-selector";
import {
  MONITOR_THUMBNAIL_INTERVAL_MS,
  refreshMonitorThumbnails,
} from "./monitor-thumbnails";
import { useRecordingSourceStore } from "./store";
import { MonitorDetails, SelectorState, WindowDetails } from "./types";
import { WindowSelector } from "./window-selector";

const clearSelectorFocus = () => {
  if (document.activeElement instanceof HTMLElement) {
    document.activeElement.blur();
  }
};

export function RecordingSourceSelectorWindow() {
  const [monitors, setMonitors] = useState<MonitorDetails[]>([]);
  const [selectorState, setSelectorState] = useState<SelectorState>({
    expanded: false,
    focusContents: false,
    placement: "above",
    revision: 0,
    windowSelector: false,
  });
  const selectorStateRef = useRef(selectorState);
  const focusedRevisionRef = useRef<number | null>(null);
  const revealedRevisionRef = useRef<number | null>(null);
  const [windows, setWindows] = useState<WindowDetails[]>([]);
  const [windowsError, setWindowsError] = useState<string | null>(null);
  const [windowsLoading, setWindowsLoading] = useState(false);
  const {
    monitorThumbnails,
    selectedMonitor,
    selectedWindow,
    setSelectedMonitor,
    setSelectedWindow,
  } = useRecordingSourceStore((state) => state);
  const { expanded: isExpanded, focusContents, windowSelector } = selectorState;

  const refreshMonitors = useCallback(async () => {
    // The tiles draw what is on each display, so the stills are recaptured
    // whenever the arrangement is; a failed capture leaves the plain fill.
    void refreshMonitorThumbnails();
    const available = await listMonitors();
    setMonitors(available);
    const { selectedMonitor, setSelectedMonitor } =
      useRecordingSourceStore.getState();
    const current = findCurrentMonitor(available, selectedMonitor);
    if (
      current &&
      JSON.stringify(current) !== JSON.stringify(selectedMonitor)
    ) {
      setSelectedMonitor(current);
    }
  }, []);

  const refreshWindows = useCallback(async () => {
    setWindowsLoading(true);
    setWindowsError(null);
    try {
      const available = await listWindows();
      setWindows(available);
      const { selectedWindow, setSelectedWindow } =
        useRecordingSourceStore.getState();
      if (
        selectedWindow &&
        !available.some((window) => window.id === selectedWindow.id)
      ) {
        setSelectedWindow(null);
      }
    } catch (error) {
      setWindowsError(
        error instanceof Error ? error.message : "Could not list windows",
      );
    } finally {
      setWindowsLoading(false);
    }
  }, []);

  useEffect(() => {
    void refreshMonitors();
  }, [refreshMonitors]);

  useEffect(() => {
    let disposed = false;
    let unlistenState: UnlistenFn | undefined;

    const applyState = (state: SelectorState) => {
      if (state.revision < selectorStateRef.current.revision) return;
      const becameExpanded =
        state.expanded && !selectorStateRef.current.expanded;
      selectorStateRef.current = state;
      setSelectorState(state);
      if (!becameExpanded) return;
      if (state.windowSelector) {
        void refreshWindows();
      } else {
        void refreshMonitors();
      }
    };

    const initialize = async () => {
      unlistenState = await listen<SelectorState>(
        "recording-source-selector://state",
        ({ payload }) => {
          applyState(payload);
        },
      );
      applyState(await getRecordingSourceSelectorState());

      if (disposed) {
        unlistenState();
      }
    };

    void initialize();

    return () => {
      disposed = true;
      unlistenState?.();
    };
  }, [refreshMonitors, refreshWindows]);

  // The popover is expanded offscreen and only ordered onscreen once this
  // webview has painted the mode it was expanded for. Revealing on a mismatch
  // is what made the previous mode's selector flash; the store sync arrives as
  // a storage event, which re-runs this effect once the mode catches up.

  useEffect(() => {
    if (!isExpanded || windowSelector) return;

    const interval = window.setInterval(() => {
      void refreshMonitors();
    }, MONITOR_THUMBNAIL_INTERVAL_MS);

    return () => {
      window.clearInterval(interval);
    };
  }, [isExpanded, refreshMonitors, windowSelector]);

  useEffect(() => {
    if (
      !isExpanded ||
      !windowSelector ||
      windowsLoading ||
      revealedRevisionRef.current === selectorState.revision
    ) {
      return;
    }

    const frame = window.requestAnimationFrame(() => {
      const target = document.querySelector<HTMLElement>(
        '[data-source-selector-focus-target="true"]',
      );
      if (!target) return;

      target.scrollIntoView({
        block: "nearest",
        inline: "nearest",
      });
      revealedRevisionRef.current = selectorState.revision;
    });

    return () => {
      window.cancelAnimationFrame(frame);
    };
  }, [
    isExpanded,
    selectorState.revision,
    windowSelector,
    windows,
    windowsLoading,
  ]);

  useEffect(() => {
    if (!isExpanded) {
      clearSelectorFocus();
      return;
    }
    if (!focusContents || focusedRevisionRef.current === selectorState.revision)
      return;

    const frame = window.requestAnimationFrame(() => {
      const target = document.querySelector<HTMLElement>(
        '[data-source-selector-focus-target="true"]',
      );
      if (!target) return;
      target.focus();
      focusedRevisionRef.current = selectorState.revision;
    });

    return () => {
      window.cancelAnimationFrame(frame);
    };
  }, [
    focusContents,
    isExpanded,
    monitors,
    selectorState.revision,
    windowSelector,
    windows,
  ]);

  // Collapsed, the webview holds a spinner rather than the last selector, so
  // the next expand shows the spinner until the requested selector paints
  // instead of flashing stale content.
  const showsWindows = isExpanded && windowSelector;
  const showsMonitors = isExpanded && !windowSelector;

  return (
    <main className="window-surface fixed inset-0 flex overflow-hidden text-content-fg">
      {showsWindows ? (
        <div className="min-h-0 grow overflow-hidden">
          <WindowSelector
            error={windowsError}
            isLoading={windowsLoading}
            onSelect={(window, returnFocus) => {
              setSelectedWindow(window);
              clearSelectorFocus();
              void collapseRecordingSourceSelector(returnFocus);
            }}
            selectedWindow={selectedWindow}
            windows={windows}
          />
        </div>
      ) : showsMonitors ? (
        <div className="flex min-h-0 grow items-center justify-center overflow-hidden">
          <MonitorSelector
            focusContents={focusContents}
            monitors={monitors}
            onCommit={(_monitor, returnFocus) => {
              clearSelectorFocus();
              void collapseRecordingSourceSelector(returnFocus);
            }}
            onSelect={(monitor) => {
              setSelectedMonitor(monitor);
            }}
            selectedMonitor={selectedMonitor}
            thumbnails={monitorThumbnails}
          />
        </div>
      ) : (
        <div className="flex grow items-center justify-center">
          <CircularProgress aria-label="Loading" isIndeterminate />
        </div>
      )}
    </main>
  );
}
