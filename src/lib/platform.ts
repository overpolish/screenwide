// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

export type Platform = "macos" | "windows" | "linux";

/** The system the app runs on, for behaviour that follows the system rather
 * than the skin: modifier keys, shortcuts, native panels and the window's own
 * buttons. Known from the user agent, so it holds at module scope too. */
export const hostPlatform: Platform =
  typeof navigator === "undefined"
    ? "macos"
    : navigator.userAgent.includes("Windows")
      ? "windows"
      : navigator.userAgent.includes("Mac")
        ? "macos"
        : "linux";

export const isMacHost = hostPlatform === "macos";
export const isWindowsHost = hostPlatform === "windows";

/** Dresses the document in `platform`'s skin: the token layer keyed off
 * `data-platform`. `main.tsx` stamps the host's; Storybook's toolbar stamps
 * either. */
export const stampPlatform = (platform: Platform) => {
  if (platform === "windows") {
    document.documentElement.dataset.platform = "windows";
  } else {
    delete document.documentElement.dataset.platform;
  }
};

/** Whether the document wears the Windows skin, which a story can switch. */
export const isWindows = () =>
  document.documentElement.dataset.platform === "windows";
