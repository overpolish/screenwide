// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

export type Platform = "macos" | "windows";

/**
 * The platform this build runs on, read the same way `main.tsx` reads it
 * before stamping `data-platform`. For structure that differs per platform
 * (which window buttons exist, what a title bar says); the skin itself is
 * CSS on `data-platform`, which Storybook can toggle live.
 */
export const detectedPlatform: Platform = navigator.userAgent.includes(
  "Windows",
)
  ? "windows"
  : "macos";
