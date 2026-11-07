// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke, isTauri } from "@tauri-apps/api/core";

/** Windows whose page is told what to do as soon as it is ready, and so says
 * it is ready itself, once it is listening, rather than on its first render. */
export const WINDOWS_ANNOUNCED_BY_PAGE = new Set(["region-selector"]);

/** Tells Rust the page is ready, so a window built for it is shown full
 * rather than empty, and what asked for it can go on. */
export const announceWindowReady = () => {
  if (isTauri()) void invoke("window_ready").catch(() => undefined);
};
