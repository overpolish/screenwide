// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { useEffect } from "react";

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

    // `listen` receives events for any target, so each window must match the
    // shortcut action it owns exactly.
    void Promise.all([
      listen<ShortcutAction>(SHORTCUT_ACTION_EVENT, ({ payload }) => {
        if (!isScreenshotShortcut(payload)) return;
        handleScreenshotShortcut(payload).catch((error: unknown) => {
          console.error("Could not open the region for a screenshot", error);
        });
      }),
      listen<ShortcutAction>(
        SCREENSHOT_SHORTCUT_REQUESTED_EVENT,
        ({ payload }) => {
          handoffScreenshotShortcut(payload).catch((error: unknown) => {
            console.error("Could not hand off the screenshot shortcut", error);
          });
        },
      ),
    ]).then((listeners) => {
      if (disposed) {
        listeners.forEach((listener) => {
          listener();
        });
      } else {
        unlisten = () => {
          listeners.forEach((listener) => {
            listener();
          });
        };
      }
    });

    return () => {
      disposed = true;
      unlisten?.();
    };
  }, [enabled]);
}
