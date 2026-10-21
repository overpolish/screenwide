// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type ReactNode, useCallback, useEffect, useState } from "react";

import { SidebarNav } from "../../composites/sidebar-nav/sidebar-nav";
import { WindowHeader } from "../../composites/window-header/window-header";
import { WindowShell } from "../../composites/window-shell/window-shell";
import { t } from "../../i18n/i18n";
import { Alert } from "../../primitives/alert/alert";
import { ScrollArea } from "../../primitives/scroll-area/scroll-area";

import { SettingsSearchField } from "./search/settings-search-field";
import { SettingsSearchResults } from "./search/settings-search-results";
import { useSettingsSearch } from "./search/use-settings-search";
import { useAnnotateSettingsSave } from "./sections/use-annotate-settings-save";
import { useGlideSettingsSave } from "./sections/use-glide-settings-save";
import { useMomentSettings } from "./sections/use-moment-settings";
import { useOcrSettingsSave } from "./sections/use-ocr-settings-save";
import { useRulerSettingsSave } from "./sections/use-ruler-settings-save";
import { useTranscription } from "./sections/use-transcription";
import { useSettingsApi } from "./settings-api-context";
import { SettingsPanes } from "./settings-panes";
import { sectionTitle, type SettingsSection } from "./settings-sections";
import { settingsSidebarItems } from "./settings-sidebar-items";
import { useShortcutCapture } from "./use-shortcut-capture";

import type { AnnotateSettings } from "../../bindings/AnnotateSettings";
import type { GeneralSettings } from "../../bindings/GeneralSettings";
import type { GlideSettings } from "../../bindings/GlideSettings";
import type { OcrSettings } from "../../bindings/OcrSettings";
import type { RulerSettings } from "../../bindings/RulerSettings";
import type { ShortcutAction } from "../../bindings/ShortcutAction";
import type { ShortcutDefaults } from "../../bindings/ShortcutDefaults";
import type { ShortcutSettings } from "../../bindings/ShortcutSettings";
import type { TranscriptionModel } from "../transcription/types";

const NO_MODELS: readonly TranscriptionModel[] = [];

export function SettingsWindow({
  initialQuery,
  initialSection = "general",
  updateSetting,
}: {
  initialQuery?: string;
  initialSection?: SettingsSection;
  updateSetting?: ReactNode;
}) {
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
    hideSettings,
    listenToAnnotateSettings,
    minimize,
    setAnnotateSettings,
    setGeneralSettings,
    setGlideSettings,
    setOcrSettings,
    setShortcutBinding,
  } = useSettingsApi();
  const [section, setSection] = useState<SettingsSection>(initialSection);
  const [general, setGeneral] = useState<GeneralSettings | null>(null);
  const [glide, setGlide] = useState<GlideSettings | null>(null);
  const [ruler, setRuler] = useState<RulerSettings | null>(null);
  const [ocr, setOcr] = useState<OcrSettings | null>(null);
  const [annotate, setAnnotate] = useState<AnnotateSettings | null>(null);
  const [defaults, setDefaults] = useState<ShortcutDefaults | null>(null);

  const [savingGeneral, setSavingGeneral] = useState(false);
  const [settings, setSettings] = useState<ShortcutSettings | null>(null);
  const [saving, setSaving] = useState<ShortcutAction | null>(null);
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
  const search = useSettingsSearch({
    initialQuery,
    models: transcription.state?.models ?? NO_MODELS,
    setSection,
  });

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
          setSettings(shortcutSettings);
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

  // The toolbar writes the same settings while the overlay is up - the tool,
  // the colour, where it was dropped - so this page follows every write rather
  // than keeping the copy it opened with, which its next edit would write back
  // over the toolbar's.
  useEffect(() => {
    let stopped = false;
    let stop: (() => void) | undefined;
    listenToAnnotateSettings(setAnnotate)
      .then((unlisten) => {
        if (stopped) unlisten();
        else stop = unlisten;
      })
      .catch((reason: unknown) => {
        setError(String(reason));
      });
    return () => {
      stopped = true;
      stop?.();
    };
  }, [listenToAnnotateSettings]);

  const changeBinding = useCallback(
    (action: ShortcutAction, shortcut: string | null) => {
      setSaving(action);
      setError(null);
      setShortcutBinding(action, shortcut)
        .then(setSettings)
        .catch((reason: unknown) => {
          setError(String(reason));
        })
        .finally(() => {
          setSaving(null);
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

  return (
    <WindowShell
      header={
        <WindowHeader
          onClose={() => void hideSettings()}
          onMinimize={() => void minimize()}
          title={
            search.isActive
              ? t("settings-search-results")
              : sectionTitle(section)
          }
        />
      }
    >
      {/* Only the sidebar keeps the window inset; the content column runs to
          the window's end and bottom edges so its scroll fade lands there,
          and carries the inset on the scrolled content instead. */}
      <div className="flex min-h-0 grow gap-group ps-inset">
        <div className="flex shrink-0 flex-col gap-separate pb-inset">
          <SettingsSearchField search={search} />
          <SidebarNav
            aria-label={t("settings-sections")}
            isExpandable={false}
            isExpanded
            items={settingsSidebarItems()}
            onSelectionChange={(id) => {
              search.setQuery("");
              setSection(id as SettingsSection);
            }}
            selected={section}
          />
        </div>
        <div className="flex min-h-0 min-w-0 grow flex-col gap-separate">
          <ScrollArea
            inWindowCorner
            key={search.isActive ? "search" : section}
            rootClassName="min-h-0 min-w-0 grow"
          >
            {search.isActive ? (
              <div className="flex flex-col pe-inset pb-inset">
                <SettingsSearchResults search={search} />
              </div>
            ) : (
              <section
                aria-label={sectionTitle(section)}
                className="flex flex-col pe-inset pb-inset"
                ref={search.paneRef}
              >
                <SettingsPanes
                  annotate={annotate}
                  defaults={defaults}
                  general={general}
                  glide={glide}
                  moments={moments}
                  ocr={ocr}
                  onCaptureChange={onCaptureChange}
                  onChangeAnnotate={changeAnnotate}
                  onChangeBinding={changeBinding}
                  onChangeGeneral={changeGeneral}
                  onChangeGlide={changeGlide}
                  onChangeOcr={changeOcr}
                  onChangeRuler={(next) => {
                    void changeRuler(next);
                  }}
                  onError={setError}
                  ruler={ruler}
                  savingAnnotate={savingAnnotate}
                  savingGeneral={savingGeneral}
                  savingGlide={savingGlide}
                  savingOcr={savingOcr}
                  savingRuler={savingRuler}
                  savingShortcut={saving}
                  section={section}
                  shortcuts={settings}
                  transcription={transcription}
                  updateSetting={updateSetting}
                />
              </section>
            )}
          </ScrollArea>
          {error ? (
            <Alert className="me-inset mb-inset" color="error" role="alert">
              {error}
            </Alert>
          ) : null}
        </div>
      </div>
    </WindowShell>
  );
}
