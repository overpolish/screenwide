// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { frameSizePreset } from "../../tools/frame/frame-sizes";

import type { ClearArea } from "../../../../bindings/ClearArea";
import type { PlatformOverlay } from "../../../../bindings/PlatformOverlay";

// What the short video apps draw over a video, as shares of a 9:16 frame:
// the header over the top, the caption, sound line and buttons over the
// bottom, and the button rail down the end side, which both sides keep so
// the screen stays centred. Measured against the apps in 2026
// (https://slovo.video/tiktok-reels-shorts-safe-zones/); feed and landscape
// platforms draw nothing over the picture. The twin of `ClearArea` in
// `src-tauri/crates/scenes/src/clear_area.rs`.
const COVERED: Record<PlatformOverlay, ClearArea> = {
  "instagram-reels": { bottom: 0.21, side: 0.05, top: 0.13 },
  tiktok: { bottom: 0.25, side: 0.05, top: 0.07 },
  "youtube-shorts": { bottom: 0.19, side: 0.05, top: 0.09 },
};

/** What the platforms published from one canvas cover between them, each
 * edge as much as the one that covers most; undefined where none covers
 * anything. */
export function platformClearArea(
  platforms: readonly string[],
): ClearArea | undefined {
  const covered = platforms.flatMap((id) => {
    const overlay = frameSizePreset(id)?.overlay;
    return overlay ? [COVERED[overlay]] : [];
  });
  if (covered.length === 0) return undefined;
  return {
    bottom: Math.max(...covered.map((area) => area.bottom)),
    side: Math.max(...covered.map((area) => area.side)),
    top: Math.max(...covered.map((area) => area.top)),
  };
}
