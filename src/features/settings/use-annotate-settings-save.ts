// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useCallback, useState } from "react";

import type { AnnotateSettings } from "./types";

export function useAnnotateSettingsSave({
  getSettings,
  saveSettings,
  setError,
  setSettings,
}: {
  getSettings: () => Promise<AnnotateSettings>;
  saveSettings: (settings: AnnotateSettings) => Promise<AnnotateSettings>;
  setError: (error: string | null) => void;
  setSettings: (settings: AnnotateSettings) => void;
}) {
  const [saving, setSaving] = useState(false);
  const change = useCallback(
    (next: AnnotateSettings) => {
      setSettings(next);
      setSaving(true);
      setError(null);
      // Rust answers with what it settled on, which is how a refused colour or
      // width puts the control back rather than leaving it showing a value the
      // overlay never took.
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
