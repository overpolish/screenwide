// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useCallback, useEffect, useRef, useState } from "react";

import { useTauriEvent } from "../../../lib/use-tauri-event";
import {
  dismissRecordingFailure,
  getRecordingFailure,
  RECORDING_FAILURES_EVENT,
} from "../api";

import type { IslandFailure } from "./island-types";

/**
 * The failure the island says, and OK to put it away. Rust keeps them in
 * turn, so none is lost to a page that was not listening yet; this asks for
 * the one in front each time Rust says it changed, and once on mounting.
 */
export function useRecordingFailure() {
  const [failure, setFailure] = useState<IslandFailure | null>(null);
  // Only the latest ask is kept, should two answers cross.
  const askedRef = useRef(0);
  const refresh = useCallback(() => {
    const asked = ++askedRef.current;
    getRecordingFailure()
      .then((current) => {
        if (asked === askedRef.current) setFailure(current);
      })
      .catch((error: unknown) => {
        console.error("Could not read the recording failure", error);
      });
  }, []);
  useEffect(refresh, [refresh]);
  useTauriEvent(RECORDING_FAILURES_EVENT, refresh);

  const dismiss = (id: number) => {
    dismissRecordingFailure(id).catch((error: unknown) => {
      console.error("Could not dismiss the recording failure", error);
    });
  };

  return { dismiss, failure };
}
