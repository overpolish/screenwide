// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useRef, useState } from "react";

import { useMeasuredHeight } from "../../lib/use-measured-height";

import {
  getRecordingOptionsState,
  RecordingOptionsState,
  setRecordingOptionsContentHeight,
} from "./api";

const CLOSED_STATE: RecordingOptionsState = {
  focusContents: false,
  open: false,
  revision: 0,
};

export function useRecordingOptionsWindowLifecycle() {
  const [popoverState, setPopoverState] =
    useState<RecordingOptionsState>(CLOSED_STATE);
  const popoverStateRef = useRef(popoverState);
  const focusedRevisionRef = useRef<number | null>(null);
  const optionsRef = useMeasuredHeight<HTMLDivElement>(
    useCallback((height: number) => {
      void setRecordingOptionsContentHeight(height).catch((error: unknown) => {
        console.error("Could not fit the recording options window", error);
      });
    }, []),
  );

  useEffect(() => {
    let disposed = false;
    let unlistenState: (() => void) | undefined;

    const applyState = (state: RecordingOptionsState) => {
      if (state.revision < popoverStateRef.current.revision) return;
      popoverStateRef.current = state;
      setPopoverState(state);
    };
    const initialize = async () => {
      unlistenState = await listen<RecordingOptionsState>(
        "recording-options://state",
        ({ payload }) => {
          applyState(payload);
        },
      );
      if (disposed) {
        unlistenState();
        return;
      }
      applyState(await getRecordingOptionsState());
    };

    void initialize();
    return () => {
      disposed = true;
      unlistenState?.();
    };
  }, []);

  useEffect(() => {
    if (!popoverState.open) {
      if (document.activeElement instanceof HTMLElement) {
        document.activeElement.blur();
      }
      return;
    }
    if (
      !popoverState.focusContents ||
      focusedRevisionRef.current === popoverState.revision
    ) {
      return;
    }

    const frame = window.requestAnimationFrame(() => {
      const target = document.querySelector<HTMLElement>(
        "[data-control-size] button:not([disabled])",
      );
      if (!target) return;
      target.focus();
      focusedRevisionRef.current = popoverState.revision;
    });
    return () => {
      window.cancelAnimationFrame(frame);
    };
  }, [popoverState]);

  return { isOpen: popoverState.open, optionsRef };
}
