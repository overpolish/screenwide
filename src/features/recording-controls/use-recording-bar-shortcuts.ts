// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { useEffect } from "react";

import {
  reportShortcutDiagnostic,
  trackShortcutListener,
} from "../../lib/shortcut-diagnostics";
import { hideRecordingUi, toggleRecordingUi } from "../recording-sources/api";
import { ShortcutAction } from "../settings/types";
import { startTextRecognition } from "../text-recognition/api";

import { startRecording } from "./api";
import { startRecordingOptions } from "./recording-request";

const SHORTCUT_ACTION_EVENT = "global-shortcut://action";

/**
 * Carries out the shortcut actions the recording bar owns.
 *
 * Emitting to a window does not scope delivery: `listen` registers for any
 * target, so every window sees every shortcut action and this listener has to
 * match the ones it owns exactly.
 */
export function useRecordingBarShortcuts() {
  useEffect(() => {
    let unlisten: UnlistenFn | undefined;
    let disposed = false;
    // Diagnostics probe (duplicate-delivery investigation): this listener
    // acknowledged nothing, so a third of all dispatches left no trace past
    // `frontend_dispatch`. The handlers below are unchanged; only the
    // reporting around them is new.
    const listener = trackShortcutListener();
    const { listenerId } = listener;

    void listen<ShortcutAction>(SHORTCUT_ACTION_EVENT, ({ payload }) => {
      if (payload === "toggleRecordingBar") {
        reportShortcutDiagnostic("received", { action: payload, listenerId });
        toggleRecordingUi().then(
          () => {
            reportShortcutDiagnostic("completed", {
              action: payload,
              listenerId,
            });
          },
          (error: unknown) => {
            reportShortcutDiagnostic("failed", {
              action: payload,
              error,
              listenerId,
            });
          },
        );
        return;
      }
      if (payload === "recognizeText") {
        reportShortcutDiagnostic("received", { action: payload, listenerId });
        void (async () => {
          await hideRecordingUi();
          await startTextRecognition();
        })()
          .then(() => {
            reportShortcutDiagnostic("completed", {
              action: payload,
              listenerId,
            });
          })
          .catch((error: unknown) => {
            reportShortcutDiagnostic("failed", {
              action: payload,
              error,
              listenerId,
            });
            console.error("Could not start text recognition", error);
          });
        return;
      }
      if (payload !== "startStopRecording") return;
      reportShortcutDiagnostic("received", { action: payload, listenerId });
      startRecording(startRecordingOptions())
        .then(() => {
          reportShortcutDiagnostic("completed", {
            action: payload,
            listenerId,
          });
        })
        .catch((error: unknown) => {
          reportShortcutDiagnostic("failed", {
            action: payload,
            error,
            listenerId,
          });
          console.error("Could not start the recording", error);
        });
    })
      .then((handle) => {
        if (disposed) handle();
        else {
          unlisten = handle;
          reportShortcutDiagnostic("listenerReady", { listenerId });
        }
      })
      .catch((error: unknown) => {
        listener.release();
        reportShortcutDiagnostic("listenerFailed", { error });
      });

    return () => {
      disposed = true;
      unlisten?.();
      listener.release();
      reportShortcutDiagnostic("listenerStopped", { listenerId });
    };
  }, []);
}
