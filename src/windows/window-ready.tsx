// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { isTauri } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect } from "react";

import {
  announceWindowReady,
  WINDOWS_ANNOUNCED_BY_PAGE,
} from "../lib/announce-window-ready";

/** Says the page has rendered, after the first commit rather than on an
 * animation frame: a hidden webview may not run frames until it is shown. */
export function WindowReady() {
  useEffect(() => {
    if (!isTauri()) return;
    if (WINDOWS_ANNOUNCED_BY_PAGE.has(getCurrentWindow().label)) return;
    announceWindowReady();
  }, []);
  return null;
}
