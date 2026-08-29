// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { useCallback, useEffect, useRef, useState } from "react";

import {
  collapseRecordingSourceSelector,
  getRecordingSourceSelectorState,
  listMonitors,
  listWindows,
} from "./api";
import { findCurrentMonitor } from "./monitor-selection";
import { MonitorSelector } from "./monitor-selector";
import { useRecordingSourceStore } from "./store";
import { MonitorDetails, SelectorState, WindowDetails } from "./types";
import { WindowSelector } from "./window-selector";

const REFRESH_INTERVAL_MS = 1_500;

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
  });
  const selectorStateRef = useRef(selectorState);
  const focusedRevisionRef = useRef<number | null>(null);
  const revealedRevisionRef = useRef<number | null>(null);
  const [windows, setWindows] = useState<WindowDetails[]>([]);
  const [windowsError, setWindowsError] = useState<string | null>(null);
  const [windowsLoading, setWindowsLoading] = useState(false);
  const {
    recordingMode,
    selectedMonitor,
    selectedWindow,
    setSelectedMonitor,
    setSelectedWindow,
  } = useRecordingSourceStore((state) => state);
  const { expanded: isExpanded, focusContents } = selectorState;

  const refreshMonitors = useCallback(async () => {
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
      if (useRecordingSourceStore.getState().recordingMode === "window") {
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

  useEffect(() => {
    if (!isExpanded || recordingMode === "window") return;

    const interval = window.setInterval(() => {
      void refreshMonitors();
    }, REFRESH_INTERVAL_MS);

    return () => {
      window.clearInterval(interval);
    };
  }, [isExpanded, recordingMode, refreshMonitors]);

  useEffect(() => {
    if (
      !isExpanded ||
      recordingMode !== "window" ||
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
    recordingMode,
    selectorState.revision,
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
    recordingMode,
    selectorState.revision,
    windows,
  ]);

  return (
    <main className="window-surface p-section fixed inset-0 flex overflow-hidden text-content-fg">
      {recordingMode === "window" ? (
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
      ) : recordingMode === "screen" ? (
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
          />
        </div>
      ) : null}
    </main>
  );
}
