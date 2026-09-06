// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { useCallback, useEffect, useState } from "react";

import { getGeneralSettings, setGeneralSettings } from "./api";
import { GeneralSettings } from "./types";

const SETTINGS_CHANGED_EVENT = "settings://changed";

/**
 * The general settings as they currently stand, and a way to change one of
 * them from a window that is not the settings window.
 *
 * A change is shown at once and stored right after; the store broadcasts what
 * it accepted, so every other window follows without asking. `null` until the
 * first load answers, and changes made before then are ignored.
 */
export function useEditableGeneralSettings() {
  const [settings, setSettings] = useState<GeneralSettings | null>(null);

  useEffect(() => {
    let unlisten: UnlistenFn | undefined;
    let disposed = false;

    void getGeneralSettings().then((loaded) => {
      if (!disposed) setSettings(loaded);
    });
    void listen<GeneralSettings>(SETTINGS_CHANGED_EVENT, ({ payload }) => {
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

  const apply = useCallback(
    (changes: Partial<GeneralSettings>) => {
      if (!settings) return;
      const next = { ...settings, ...changes };
      setSettings(next);
      setGeneralSettings(next)
        .then(setSettings)
        .catch((cause: unknown) => {
          console.error("Could not store the general settings", cause);
          void getGeneralSettings().then(setSettings);
        });
    },
    [settings],
  );

  return [settings, apply] as const;
}

/** The general settings, for windows that only read them. */
export function useGeneralSettings(): GeneralSettings | null {
  return useEditableGeneralSettings()[0];
}
