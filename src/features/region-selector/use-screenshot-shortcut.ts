// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { useEffect } from "react";

import { useRecordingSourceStore } from "../recording-sources/store";
import { ShortcutAction } from "../settings/types";

import {
  handleScreenshotShortcut,
  handoffScreenshotShortcut,
  isScreenshotShortcut,
} from "./screenshot-session";
import { reportShortcutDiagnostic } from "./shortcut-diagnostics";

const SHORTCUT_ACTION_EVENT = "global-shortcut://action";
const SCREENSHOT_SHORTCUT_REQUESTED_EVENT =
  "screenshot-region://shortcut-requested";

export function useScreenshotShortcut(enabled = true) {
  useEffect(() => {
    if (!enabled) return;

    let unlisten: UnlistenFn | undefined;
    let disposed = false;

    // `listen` receives events for any target, so each window must match the
    // shortcut action it owns exactly.
    void Promise.all([
      listen<ShortcutAction>(SHORTCUT_ACTION_EVENT, ({ payload }) => {
        if (!isScreenshotShortcut(payload)) return;
        // Rust routes active sessions through the dedicated handoff event.
        // Tauri listeners can still observe the original targeted event; do
        // not let that duplicate turn a destination switch into a dismissal.
        reportShortcutDiagnostic("received", payload);
        if (useRecordingSourceStore.getState().isScreenshotCapture) {
          reportShortcutDiagnostic("duplicateIgnored", payload);
          return;
        }
        handleScreenshotShortcut(payload)
          .then(() => {
            reportShortcutDiagnostic("completed", payload);
          })
          .catch((error: unknown) => {
            reportShortcutDiagnostic("failed", payload, error);
            console.error("Could not open the region for a screenshot", error);
          });
      }),
      listen<ShortcutAction>(
        SCREENSHOT_SHORTCUT_REQUESTED_EVENT,
        ({ payload }) => {
          reportShortcutDiagnostic("received", payload);
          handoffScreenshotShortcut(payload)
            .then(() => {
              reportShortcutDiagnostic("completed", payload);
            })
            .catch((error: unknown) => {
              reportShortcutDiagnostic("failed", payload, error);
              console.error(
                "Could not hand off the screenshot shortcut",
                error,
              );
            });
        },
      ),
    ])
      .then((listeners) => {
        if (disposed) {
          listeners.forEach((listener) => {
            listener();
          });
        } else {
          reportShortcutDiagnostic("listenerReady");
          unlisten = () => {
            listeners.forEach((listener) => {
              listener();
            });
          };
        }
      })
      .catch((error: unknown) => {
        reportShortcutDiagnostic("listenerFailed", undefined, error);
      });

    return () => {
      reportShortcutDiagnostic("listenerStopped");
      disposed = true;
      unlisten?.();
    };
  }, [enabled]);
}
