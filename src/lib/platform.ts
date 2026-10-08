// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/** Read from the attribute `main.tsx` sets, so a story can switch platforms
 * by setting it too. */
export const isWindows = () =>
  document.documentElement.dataset.platform === "windows";

/** Where a deleted project goes, by the name this platform gives it. */
export const trashName = () => (isWindows() ? "Recycle Bin" : "Trash");

/** The file manager a project is shown in. */
export const fileManagerName = () => (isWindows() ? "Explorer" : "Finder");
