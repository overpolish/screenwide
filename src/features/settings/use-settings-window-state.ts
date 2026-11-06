// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useCallback, useEffect, useState } from "react";

import { useLicences } from "./licences/use-licences";
import { useAnnotateSettingsSave } from "./sections/use-annotate-settings-save";
import { useGlideSettingsSave } from "./sections/use-glide-settings-save";
import { useMomentSettings } from "./sections/use-moment-settings";
import { useOcrSettingsSave } from "./sections/use-ocr-settings-save";
import { useRulerSettingsSave } from "./sections/use-ruler-settings-save";
import { useTranscription } from "./sections/use-transcription";
import { useSettingsApi } from "./settings-api-context";
import { useShortcutCapture } from "./use-shortcut-capture";

import type { AnnotateSettings } from "../../bindings/AnnotateSettings";
import type { GeneralSettings } from "../../bindings/GeneralSettings";
import type { GlideSettings } from "../../bindings/GlideSettings";
import type { OcrSettings } from "../../bindings/OcrSettings";
import type { RulerSettings } from "../../bindings/RulerSettings";
import type { ShortcutAction } from "../../bindings/ShortcutAction";
import type { ShortcutDefaults } from "../../bindings/ShortcutDefaults";
import type { ShortcutSettings } from "../../bindings/ShortcutSettings";

// Every setting the window shows, read together as it opens and written
// back as each changes. A change shows at once; one the app refuses puts
// back what it has and says why in the window's alert.

export function useSettingsWindowState() {
  const {
    beginShortcutCapture,
    endShortcutCapture,
    getAnnotateSettings,
    getGeneralSettings,
    getGlideSettings,
    getOcrSettings,
    getRulerSettings,
    getShortcutDefaults,
    getShortcutSettings,
    listenToAnnotateSettings,
    listenToGeneralSettings,
    setAnnotateSettings,
    setGeneralSettings,
    setGlideSettings,
    setOcrSettings,
    setShortcutBinding,
  } = useSettingsApi();
  const [general, setGeneral] = useState<GeneralSettings | null>(null);
  const [glide, setGlide] = useState<GlideSettings | null>(null);
  const [ruler, setRuler] = useState<RulerSettings | null>(null);
  const [ocr, setOcr] = useState<OcrSettings | null>(null);
  const [annotate, setAnnotate] = useState<AnnotateSettings | null>(null);
  const [defaults, setDefaults] = useState<ShortcutDefaults | null>(null);
  const [savingGeneral, setSavingGeneral] = useState(false);
  const [shortcuts, setShortcuts] = useState<ShortcutSettings | null>(null);
  const [savingShortcut, setSavingShortcut] = useState<ShortcutAction | null>(
    null,
  );
  const [error, setError] = useState<string | null>(null);
  const { changeRuler, savingRuler } = useRulerSettingsSave(setRuler, setError);
  const onCaptureChange = useShortcutCapture(
    beginShortcutCapture,
    endShortcutCapture,
  );
  const { change: changeGlide, saving: savingGlide } = useGlideSettingsSave({
    reloadSettings: getGlideSettings,
    saveSettings: setGlideSettings,
    setError,
    setGlide,
  });
  const { change: changeOcr, saving: savingOcr } = useOcrSettingsSave({
    getSettings: getOcrSettings,
    saveSettings: setOcrSettings,
    setError,
    setSettings: setOcr,
  });
  const { change: changeAnnotate, saving: savingAnnotate } =
    useAnnotateSettingsSave({
      getSettings: getAnnotateSettings,
      saveSettings: setAnnotateSettings,
      setError,
      setSettings: setAnnotate,
    });
  const moments = useMomentSettings(setError);
  const transcription = useTranscription(setError);
  const licences = useLicences(setError);

  useEffect(() => {
    Promise.all([
      getGeneralSettings(),
      getGlideSettings(),
      getRulerSettings(),
      getShortcutSettings(),
      getOcrSettings(),
      getAnnotateSettings(),
      getShortcutDefaults(),
    ])
      .then(
        ([
          generalSettings,
          glideSettings,
          rulerSettings,
          shortcutSettings,
          ocrSettings,
          annotateSettings,
          shortcutDefaults,
        ]) => {
          setGeneral(generalSettings);
          setGlide(glideSettings);
          setRuler(rulerSettings);
          setShortcuts(shortcutSettings);
          setOcr(ocrSettings);
          setAnnotate(annotateSettings);
          setDefaults(shortcutDefaults);
        },
      )
      .catch((reason: unknown) => {
        setError(String(reason));
      });
  }, [
    getGeneralSettings,
    getGlideSettings,
    getRulerSettings,
    getShortcutSettings,
    getOcrSettings,
    getAnnotateSettings,
    getShortcutDefaults,
  ]);

  // Other windows write the same settings while this one is open: the
  // Annotate toolbar its tool, colour and place; the island, the editor and
  // the first launch of a newer build the general ones. The page follows
  // every write rather than keeping the copy it opened with, which its next
  // edit would write back over theirs.
  useFollowWrites(listenToAnnotateSettings, setAnnotate, setError);
  useFollowWrites(listenToGeneralSettings, setGeneral, setError);

  const changeBinding = useCallback(
    (action: ShortcutAction, shortcut: string | null) => {
      setSavingShortcut(action);
      setError(null);
      setShortcutBinding(action, shortcut)
        .then(setShortcuts)
        .catch((reason: unknown) => {
          setError(String(reason));
        })
        .finally(() => {
          setSavingShortcut(null);
        });
    },
    [setShortcutBinding],
  );

  const changeGeneral = useCallback(
    (next: GeneralSettings) => {
      setGeneral(next);
      setSavingGeneral(true);
      setError(null);
      setGeneralSettings(next)
        .then(setGeneral)
        .catch((reason: unknown) => {
          setError(String(reason));
          void getGeneralSettings().then(setGeneral);
        })
        .finally(() => {
          setSavingGeneral(false);
        });
    },
    [getGeneralSettings, setGeneralSettings],
  );

  return {
    annotate,
    defaults,
    error,
    general,
    glide,
    licences,
    moments,
    ocr,
    onCaptureChange,
    onChangeAnnotate: changeAnnotate,
    onChangeBinding: changeBinding,
    onChangeGeneral: changeGeneral,
    onChangeGlide: changeGlide,
    onChangeOcr: changeOcr,
    onChangeRuler: (next: RulerSettings) => {
      void changeRuler(next);
    },
    onError: setError,
    ruler,
    savingAnnotate,
    savingGeneral,
    savingGlide,
    savingOcr,
    savingRuler,
    savingShortcut,
    shortcuts,
    transcription,
  };
}

function useFollowWrites<T>(
  listen: (onChange: (settings: T) => void) => Promise<() => void>,
  onChange: (settings: T) => void,
  onError: (message: string) => void,
) {
  useEffect(() => {
    let stopped = false;
    let stop: (() => void) | undefined;
    listen(onChange)
      .then((unlisten) => {
        if (stopped) unlisten();
        else stop = unlisten;
      })
      .catch((reason: unknown) => {
        onError(String(reason));
      });
    return () => {
      stopped = true;
      stop?.();
    };
  }, [listen, onChange, onError]);
}
