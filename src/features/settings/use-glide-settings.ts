// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";

import { getGlideSettings } from "./api";
import { GlideSettings } from "./types";

const GLIDE_SETTINGS_CHANGED_EVENT = "glide-settings://changed";

/**
 * The Glide settings as they currently stand, for windows that only read them:
 * the stored values on mount, then whatever the settings window saves.
 * `null` until the first load answers.
 */
export function useGlideSettings(): GlideSettings | null {
  const [settings, setSettings] = useState<GlideSettings | null>(null);

  useEffect(() => {
    let unlisten: UnlistenFn | undefined;
    let disposed = false;

    void getGlideSettings().then((loaded) => {
      if (!disposed) setSettings(loaded);
    });
    void listen<GlideSettings>(GLIDE_SETTINGS_CHANGED_EVENT, ({ payload }) => {
      setSettings(payload);
    }).then((listener) => {
      if (disposed) listener();
      else unlisten = listener;
    });

    return () => {
      disposed = true;
      unlisten?.();
    };
  }, []);

  return settings;
}
