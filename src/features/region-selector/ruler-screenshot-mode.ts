// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";

/** Deferred teardown used by the persistent control-plane window. */
export const cancelRuler = () => invoke<null>("cancel_ruler");

/** Lifecycle-only handoff; Ruler rendering and state remain entirely native. */
export const setRulerScreenshotMode = (active: boolean) =>
  invoke<null>("set_ruler_screenshot_mode", { active });
