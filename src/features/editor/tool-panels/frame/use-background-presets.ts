// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke, isTauri } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";

import {
  backgroundPresetsWithWallpapers,
  BUILT_IN_BACKGROUND_PRESETS,
  SystemWallpaper,
} from "../../../../components/shared/background-picker/background-presets";

/**
 * The backgrounds the picker offers: the app's own, and the pictures on the
 * desktop.
 *
 * The desktop's pictures come from the native side, and are asked for each
 * time a panel opens because the wallpaper can change while the app runs.
 * Until they arrive, when they cannot be read, and in a story where there is
 * nothing to ask, the picker shows the palettes and the flat tones alone.
 */
export function useBuiltInBackgroundPresets() {
  const [presets, setPresets] = useState(BUILT_IN_BACKGROUND_PRESETS);

  useEffect(() => {
    if (!isTauri()) return;
    let current = true;
    void invoke<SystemWallpaper[]>("list_system_wallpapers").then(
      (found) => {
        if (current) setPresets(backgroundPresetsWithWallpapers(found));
      },
      () => {
        // The built-ins are already showing, and stay.
      },
    );
    return () => {
      current = false;
    };
  }, []);

  return presets;
}
