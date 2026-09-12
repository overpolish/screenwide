// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke, isTauri } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";

import {
  backgroundPresetsWithWallpapers,
  BUILT_IN_BACKGROUND_PRESETS,
  SystemWallpaper,
} from "../../../components/shared/background-picker/background-presets";

/** Asked for once per window: the system's desktop pictures do not move while
 * the app is running, and every panel that opens shows the same tiles. */
let wallpapers: Promise<SystemWallpaper[]> | null = null;

const systemWallpapers = () => {
  wallpapers ??= invoke<SystemWallpaper[]>("list_system_wallpapers").catch(
    () => {
      // A folder that cannot be read leaves the built-ins without wallpapers
      // rather than without a picker, and is not held, so the next panel
      // tries again.
      wallpapers = null;
      return [];
    },
  );
  return wallpapers;
};

/**
 * The backgrounds the picker offers: the app's own, and the system's.
 *
 * The wallpapers are the pictures the operating system already ships, so the
 * list of them comes from the native side. Until it arrives, and in a story
 * where there is nothing to ask, the picker shows the palettes and the flat
 * tones alone.
 */
export function useBuiltInBackgroundPresets() {
  const [presets, setPresets] = useState(BUILT_IN_BACKGROUND_PRESETS);

  useEffect(() => {
    if (!isTauri()) return;
    let current = true;
    void systemWallpapers().then((found) => {
      if (current) setPresets(backgroundPresetsWithWallpapers(found));
    });
    return () => {
      current = false;
    };
  }, []);

  return presets;
}
