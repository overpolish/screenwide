// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useCallback, useState } from "react";

import type { OcrSettings } from "./types";

export function useOcrSettingsSave({
  getSettings,
  saveSettings,
  setError,
  setSettings,
}: {
  getSettings: () => Promise<OcrSettings>;
  saveSettings: (settings: OcrSettings) => Promise<OcrSettings>;
  setError: (error: string | null) => void;
  setSettings: (settings: OcrSettings) => void;
}) {
  const [saving, setSaving] = useState(false);
  const change = useCallback(
    (next: OcrSettings) => {
      setSettings(next);
      setSaving(true);
      setError(null);
      void saveSettings(next)
        .then(setSettings)
        .catch((reason: unknown) => {
          setError(String(reason));
          void getSettings()
            .then(setSettings)
            .catch(() => undefined);
        })
        .finally(() => {
          setSaving(false);
        });
    },
    [getSettings, saveSettings, setError, setSettings],
  );
  return { change, saving };
}
