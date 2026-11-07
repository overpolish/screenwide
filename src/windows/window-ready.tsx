// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke, isTauri } from "@tauri-apps/api/core";
import { useEffect } from "react";

/** Tells Rust the page has rendered, so a window built for it is shown full
 * rather than empty. Announced after the first commit rather than on an
 * animation frame: a hidden webview may not run frames until it is shown. */
export function WindowReady() {
  useEffect(() => {
    if (isTauri()) void invoke("window_ready").catch(() => undefined);
  }, []);
  return null;
}
