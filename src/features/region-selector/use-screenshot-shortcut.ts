// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { useEffect } from "react";

import { ShortcutAction } from "../settings/types";

import {
  beginScreenshotCapture,
  isScreenshotShortcut,
} from "./screenshot-session";

const SHORTCUT_ACTION_EVENT = "global-shortcut://action";

export function useScreenshotShortcut() {
  useEffect(() => {
    let unlisten: UnlistenFn | undefined;
    let disposed = false;

    // `listen` receives events for any target, so each window must match the
    // shortcut action it owns exactly.
    void listen<ShortcutAction>(SHORTCUT_ACTION_EVENT, ({ payload }) => {
      if (!isScreenshotShortcut(payload)) return;
      beginScreenshotCapture(payload).catch((error: unknown) => {
        console.error("Could not open the region for a screenshot", error);
      });
    }).then((listener) => {
      if (disposed) listener();
      else unlisten = listener;
    });

    return () => {
      disposed = true;
      unlisten?.();
    };
  }, []);
}
