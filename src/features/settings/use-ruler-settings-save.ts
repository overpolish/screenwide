// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useCallback, useState } from "react";

import { useSettingsApi } from "./settings-api-context";

import type { RulerSettings } from "./types";

export function useRulerSettingsSave(
  setRuler: (settings: RulerSettings) => void,
  setError: (error: string | null) => void,
) {
  const { getRulerSettings, setRulerSettings } = useSettingsApi();
  const [savingRuler, setSavingRuler] = useState(false);
  const changeRuler = useCallback(
    async (next: RulerSettings) => {
      setRuler(next);
      setSavingRuler(true);
      setError(null);
      try {
        setRuler(await setRulerSettings(next));
      } catch (reason) {
        setError(String(reason));
        try {
          setRuler(await getRulerSettings());
        } catch (reloadError) {
          setError(String(reloadError));
        }
      } finally {
        setSavingRuler(false);
      }
    },
    [getRulerSettings, setRulerSettings, setRuler, setError],
  );
  return { changeRuler, savingRuler };
}
