// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { useEffect } from "react";

import {
  reportShortcutDiagnostic,
  trackShortcutListener,
} from "../../lib/shortcut-diagnostics";
import { useRecordingSourceStore } from "../recording-sources/store";
import { ShortcutAction } from "../settings/types";

import {
  handleScreenshotShortcut,
  handoffScreenshotShortcut,
  isScreenshotShortcut,
} from "./screenshot-session";

const SHORTCUT_ACTION_EVENT = "global-shortcut://action";
const SCREENSHOT_SHORTCUT_REQUESTED_EVENT =
  "screenshot-region://shortcut-requested";

export function useScreenshotShortcut(enabled = true) {
  useEffect(() => {
    if (!enabled) return;

    let unlisten: UnlistenFn | undefined;
    let disposed = false;
    // Diagnostics probe (duplicate-delivery investigation): one identity per
    // subscription, claimed before `listen` resolves, so a press handled twice
    // can be read as two subscriptions, one doubled delivery, or two webviews.
    const action = trackShortcutListener();
    const handoff = trackShortcutListener();

    // `listen` receives events for any target, so each window must match the
    // shortcut action it owns exactly.
    void Promise.all([
      listen<ShortcutAction>(SHORTCUT_ACTION_EVENT, ({ payload }) => {
        if (disposed) return;
        if (!isScreenshotShortcut(payload)) return;
        // Rust routes active sessions through the dedicated handoff event.
        // Tauri listeners can still observe the original targeted event; do
        // not let that duplicate turn a destination switch into a dismissal.
        const listenerId = action.listenerId;
        reportShortcutDiagnostic("received", { action: payload, listenerId });
        if (useRecordingSourceStore.getState().isScreenshotCapture) {
          reportShortcutDiagnostic("duplicateIgnored", {
            action: payload,
            listenerId,
          });
          return;
        }
        handleScreenshotShortcut(payload)
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
            console.error("Could not open the region for a screenshot", error);
          });
      }),
      listen<ShortcutAction>(
        SCREENSHOT_SHORTCUT_REQUESTED_EVENT,
        ({ payload }) => {
          if (disposed) return;
          const listenerId = handoff.listenerId;
          reportShortcutDiagnostic("received", { action: payload, listenerId });
          handoffScreenshotShortcut(payload)
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
          reportShortcutDiagnostic("listenerReady", {
            listenerId: action.listenerId,
          });
          reportShortcutDiagnostic("listenerReady", {
            listenerId: handoff.listenerId,
          });
          unlisten = () => {
            listeners.forEach((listener) => {
              listener();
            });
          };
        }
      })
      .catch((error: unknown) => {
        action.release();
        handoff.release();
        reportShortcutDiagnostic("listenerFailed", { error });
      });

    return () => {
      disposed = true;
      unlisten?.();
      action.release();
      handoff.release();
      reportShortcutDiagnostic("listenerStopped", {
        listenerId: action.listenerId,
      });
      reportShortcutDiagnostic("listenerStopped", {
        listenerId: handoff.listenerId,
      });
    };
  }, [enabled]);
}
