// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useCallback, useRef, useState } from "react";

import type { GlideSettings } from "./types";

export function useGlideSettingsSave({
  reloadSettings,
  saveSettings,
  setError,
  setGlide,
}: {
  reloadSettings: () => Promise<GlideSettings>;
  saveSettings: (settings: GlideSettings) => Promise<GlideSettings>;
  setError: (error: string | null) => void;
  setGlide: (settings: GlideSettings) => void;
}) {
  const [saving, setSaving] = useState(false);
  const queueRef = useRef<{ pending: GlideSettings | null; running: boolean }>({
    pending: null,
    running: false,
  });
  const change = useCallback(
    (next: GlideSettings) => {
      setGlide(next);
      setError(null);
      queueRef.current.pending = next;
      if (queueRef.current.running) return;
      queueRef.current.running = true;
      setSaving(true);
      void (async () => {
        const hasPending = () => queueRef.current.pending !== null;
        try {
          while (queueRef.current.pending) {
            const draft = queueRef.current.pending;
            queueRef.current.pending = null;
            try {
              const saved = await saveSettings(draft);
              if (!hasPending()) setGlide(saved);
            } catch (reason: unknown) {
              setError(String(reason));
              if (!hasPending()) {
                const saved = await reloadSettings();
                if (!hasPending()) setGlide(saved);
              }
            }
          }
        } catch (reason: unknown) {
          setError(String(reason));
        } finally {
          queueRef.current.running = false;
          setSaving(false);
        }
      })();
    },
    [reloadSettings, saveSettings, setError, setGlide],
  );
  return { change, saving };
}
