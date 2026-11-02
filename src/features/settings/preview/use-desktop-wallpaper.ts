// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { convertFileSrc, invoke, isTauri } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";

// The person's own desktop picture, for previews that show something on
// their desktop: drawn by the app at a preview's size into a file the page
// loads, once for every preview in the window. It is the picture every
// display shares unless one overrides it; neither system says which
// display shows which other picture. `null` until it loads, where the
// system names none, and outside the app.

/** As much as a preview can show of a desktop, in pixels. */
const SIZE = { height: 1280, width: 2048 };

let loading: Promise<HTMLImageElement | null> | null = null;

async function load() {
  const path = await invoke<string | null>("render_desktop_wallpaper", SIZE);
  if (!path) return null;
  const image = new Image();
  // Painted into pictures the page reads back, so it must not taint the
  // canvas; the asset protocol answers with the page's origin.
  image.crossOrigin = "anonymous";
  image.src = convertFileSrc(path);
  await image.decode();
  return image;
}

export function useDesktopWallpaper() {
  const [wallpaper, setWallpaper] = useState<HTMLImageElement | null>(null);

  useEffect(() => {
    if (!isTauri()) return;
    let current = true;
    loading ??= load().catch((error: unknown) => {
      console.error("The desktop picture could not be read", error);
      return null;
    });
    void loading.then((image) => {
      if (current) setWallpaper(image);
    });
    return () => {
      current = false;
    };
  }, []);

  return wallpaper;
}
