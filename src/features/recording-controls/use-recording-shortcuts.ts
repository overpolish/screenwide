// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { useEffect } from "react";

import { hideRecordingUi, toggleRecordingUi } from "../recording-sources/api";
import { startTextRecognition } from "../text-recognition/api";

import { startRecording } from "./api";
import { startRecordingOptions } from "./recording-request";

import type { ShortcutAction } from "../../bindings/ShortcutAction";

const SHORTCUT_ACTION_EVENT = "global-shortcut://action";

/**
 * Carries out the shortcut actions the recording controls own.
 *
 * Emitting to a window does not scope delivery: `listen` registers for any
 * target, so every window sees every shortcut action and this listener has to
 * match the ones it owns exactly.
 */
export function useRecordingShortcuts() {
  useEffect(() => {
    let unlisten: UnlistenFn | undefined;
    let disposed = false;

    void listen<ShortcutAction>(SHORTCUT_ACTION_EVENT, ({ payload }) => {
      if (disposed) return;
      if (payload === "toggleRecordingControls") {
        void toggleRecordingUi();
        return;
      }
      if (payload === "recognizeText") {
        void (async () => {
          await hideRecordingUi();
          await startTextRecognition();
        })().catch((error: unknown) => {
          console.error("Could not start text recognition", error);
        });
        return;
      }
      if (payload !== "startStopRecording") return;
      startRecordingOptions()
        .then(startRecording)
        .catch((error: unknown) => {
          console.error("Could not start the recording", error);
        });
    }).then((handle) => {
      if (disposed) handle();
      else unlisten = handle;
    });

    return () => {
      disposed = true;
      unlisten?.();
    };
  }, []);
}
