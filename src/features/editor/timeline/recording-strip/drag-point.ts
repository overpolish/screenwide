// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { isWindowsHost } from "../../../../lib/platform";

/** Where on the page, in CSS px, Tauri's drag event says the drag is. It
 * calls the position physical, which it is on Windows, in the window's
 * pixels; on macOS it is already in CSS px. */
export function dragPoint({ x, y }: { x: number; y: number }) {
  if (!isWindowsHost) return { x, y };
  const scale = window.devicePixelRatio;
  return { x: x / scale, y: y / scale };
}
