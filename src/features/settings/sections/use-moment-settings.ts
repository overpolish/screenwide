// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useCallback, useEffect, useState } from "react";

import {
  ANNOTATION_SWATCHES,
  sameAnnotationColor,
} from "../../../components/shared/annotation-style/palette";
import { useSettingsApi } from "../settings-api-context";

import type { MomentKind, MomentSettings } from "../types";

/** The first palette colour no kind has yet, so a new kind stands apart. */
const unusedColor = (kinds: MomentKind[]) =>
  (
    ANNOTATION_SWATCHES.find(
      (swatch) =>
        !kinds.some((kind) => sameAnnotationColor(kind.color, swatch.color)),
    ) ?? ANNOTATION_SWATCHES[0]
  ).color;

/** The kinds of moment as Rust keeps them, and a save that falls back to the
 * stored kinds when Rust refuses a change, such as a shortcut already taken. */
export function useMomentSettings(onError: (error: string | null) => void) {
  const { getMomentSettings, setMomentSettings } = useSettingsApi();
  const [settings, setSettings] = useState<MomentSettings | null>(null);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    getMomentSettings()
      .then(setSettings)
      .catch((reason: unknown) => {
        onError(String(reason));
      });
  }, [getMomentSettings, onError]);

  const save = useCallback(
    async (next: MomentSettings) => {
      setSettings(next);
      setSaving(true);
      onError(null);
      try {
        setSettings(await setMomentSettings(next));
      } catch (reason) {
        onError(String(reason));
        try {
          setSettings(await getMomentSettings());
        } catch (reloadError) {
          onError(String(reloadError));
        }
      } finally {
        setSaving(false);
      }
    },
    [getMomentSettings, onError, setMomentSettings],
  );

  const addKind = useCallback(() => {
    if (!settings) return;
    void save({
      ...settings,
      kinds: [
        ...settings.kinds,
        {
          color: unusedColor(settings.kinds),
          customColor: null,
          id: crypto.randomUUID(),
          name: "New Kind",
          shortcut: null,
        },
      ],
    });
  }, [save, settings]);

  /** Shows a colour without keeping it, for the steps of a drag through the
   * Colours panel; the colour it settles on is saved. */
  const preview = useCallback((next: MomentSettings) => {
    setSettings(next);
  }, []);

  return { addKind, isSaving: saving, preview, save, settings };
}

export type MomentSettingsState = ReturnType<typeof useMomentSettings>;
