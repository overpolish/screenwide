// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { useEffect } from "react";

import { reportShortcutDiagnostic } from "../../lib/shortcut-diagnostics";
import { useRecordingSourceStore } from "../recording-sources/store";

import {
  handleScreenshotShortcut,
  handoffScreenshotShortcut,
  isScreenshotShortcut,
  takePendingShortcutAction,
} from "./screenshot-session";

import type { ShortcutAction } from "../../bindings/ShortcutAction";

const SHORTCUT_ACTION_EVENT = "global-shortcut://action";
const SCREENSHOT_SHORTCUT_REQUESTED_EVENT =
  "screenshot-region://shortcut-requested";

/** Carries out the screenshot shortcuts. Built for one, the page takes it once
 * it is listening; `onListening` then runs, so Rust can tell it the rest. */
export function useScreenshotShortcut(onListening: () => void) {
  useEffect(() => {
    let unlisten: UnlistenFn | undefined;
    let disposed = false;

    const carryOut = (action: ShortcutAction) => {
      if (!isScreenshotShortcut(action)) return;
      // Rust routes active sessions through the dedicated handoff event.
      // Tauri listeners can still observe the original targeted event; do
      // not let that duplicate turn a destination switch into a dismissal.
      reportShortcutDiagnostic("received", action);
      if (useRecordingSourceStore.getState().isScreenshotCapture) {
        reportShortcutDiagnostic("duplicateIgnored", action);
        return;
      }
      handleScreenshotShortcut(action)
        .then(() => {
          reportShortcutDiagnostic("completed", action);
        })
        .catch((error: unknown) => {
          reportShortcutDiagnostic("failed", action, error);
          console.error("Could not open the region for a screenshot", error);
        });
    };

    // `listen` receives events for any target, so each window must match the
    // shortcut action it owns exactly.
    void Promise.all([
      listen<ShortcutAction>(SHORTCUT_ACTION_EVENT, ({ payload }) => {
        if (disposed) return;
        carryOut(payload);
      }),
      listen<ShortcutAction>(
        SCREENSHOT_SHORTCUT_REQUESTED_EVENT,
        ({ payload }) => {
          if (disposed) return;
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
          void takePendingShortcutAction()
            .then((action) => {
              if (action && !disposed) carryOut(action);
            })
            .catch(() => undefined)
            .finally(onListening);
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
  }, [onListening]);
}
