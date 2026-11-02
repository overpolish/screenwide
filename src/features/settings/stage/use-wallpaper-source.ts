// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useMemo } from "react";

import { paintWallpaper } from "../preview/scenes/scene-kit";
import { useDesktopWallpaper } from "../preview/use-desktop-wallpaper";

let painted: string | null = null;

/** The sample desktop's picture, painted once, as an image the page can
 * show. */
function paintedWallpaper() {
  if (painted !== null) return painted;
  const canvas = document.createElement("canvas");
  const size = { height: 400, width: 640 };
  canvas.width = size.width;
  canvas.height = size.height;
  const context = canvas.getContext("2d");
  if (context) paintWallpaper(context, size, null);
  painted = canvas.toDataURL("image/jpeg", 0.9);
  return painted;
}

/** Where the desktop's picture can be shown from: the person's own where
 * the app can read it, or the sample desktop's painted one. */
export function useWallpaperSource() {
  const wallpaper = useDesktopWallpaper();
  return useMemo(() => wallpaper?.src ?? paintedWallpaper(), [wallpaper]);
}
