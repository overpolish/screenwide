// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/** Read from the attribute `main.tsx` sets, so a story can switch platforms
 * by setting it too. */
export const isWindows = () =>
  document.documentElement.dataset.platform === "windows";
