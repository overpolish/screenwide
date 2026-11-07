// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useSyncExternalStore } from "react";

const watchVisibility = (onChange: () => void) => {
  document.addEventListener("visibilitychange", onChange);
  return () => {
    document.removeEventListener("visibilitychange", onChange);
  };
};
const isPageVisible = () => document.visibilityState === "visible";

/** Whether this window's page is on screen. App windows are hidden rather
 * than closed, so what a page holds only while it is seen (a microphone, a
 * native picture) follows this rather than the page's lifetime. */
export const usePageVisible = () =>
  useSyncExternalStore(watchVisibility, isPageVisible, () => false);
