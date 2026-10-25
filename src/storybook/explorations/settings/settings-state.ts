// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useCallback, useEffect, useState } from "react";

import { useMomentSettings } from "../../../features/settings/sections/use-moment-settings";
import { useTranscription } from "../../../features/settings/sections/use-transcription";
import { useSettingsApi } from "../../../features/settings/settings-api-context";

import type { ShortcutAction } from "../../../bindings/ShortcutAction";
import type { ShortcutSettings } from "../../../bindings/ShortcutSettings";

// Every setting the explored window shows, read from and written to the
// stories' in-memory settings API, so a change in one pane shows in every
// other: a tool turned off in Tools greys its keys in Shortcuts, a moment
// renamed in Moments is the name the island shows.

/** One stored setting: null until read, then written through as changed. */
function useStored<T>(
  get: () => Promise<T>,
  set: (next: T) => Promise<T>,
  onError: (error: string | null) => void,
) {
  const [value, setValue] = useState<T | null>(null);
  useEffect(() => {
    get()
      .then(setValue)
      .catch((reason: unknown) => {
        onError(String(reason));
      });
  }, [get, onError]);
  const change = useCallback(
    (next: T) => {
      setValue(next);
      set(next).catch((reason: unknown) => {
        onError(String(reason));
      });
    },
    [set, onError],
  );
  return [value, change] as const;
}

export function useExploredSettings() {
  const api = useSettingsApi();
  const [error, setError] = useState<string | null>(null);
  const [general, setGeneral] = useStored(
    api.getGeneralSettings,
    api.setGeneralSettings,
    setError,
  );
  const [glide, setGlide] = useStored(
    api.getGlideSettings,
    api.setGlideSettings,
    setError,
  );
  const [ruler, setRuler] = useStored(
    api.getRulerSettings,
    api.setRulerSettings,
    setError,
  );
  const [ocr, setOcr] = useStored(
    api.getOcrSettings,
    api.setOcrSettings,
    setError,
  );
  const [annotate, setAnnotate] = useStored(
    api.getAnnotateSettings,
    api.setAnnotateSettings,
    setError,
  );
  const [shortcuts, setShortcuts] = useState<ShortcutSettings | null>(null);
  useEffect(() => {
    void api.getShortcutSettings().then(setShortcuts);
  }, [api]);
  const setBinding = useCallback(
    (action: ShortcutAction, shortcut: string | null) => {
      void api.setShortcutBinding(action, shortcut).then(setShortcuts);
    },
    [api],
  );
  // The frame rate and anti-flicker belong to the recording inputs, not the
  // stored preferences; the exploration keeps them here.
  const [fps, setFps] = useState<30 | 60>(60);
  const [antiFlicker, setAntiFlicker] = useState(false);
  const moments = useMomentSettings(setError);
  const transcription = useTranscription(setError);

  return {
    annotate,
    antiFlicker,
    error,
    fps,
    general,
    glide,
    moments,
    ocr,
    ruler,
    setAnnotate,
    setAntiFlicker,
    setBinding,
    setFps,
    setGeneral,
    setGlide,
    setOcr,
    setRuler,
    shortcuts,
    transcription,
  };
}

export type ExploredSettings = ReturnType<typeof useExploredSettings>;
