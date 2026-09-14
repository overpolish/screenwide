// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Keyboard, LayoutGrid, Ruler, ScanText, Settings } from "lucide-react";
import { type ReactNode, useCallback, useEffect, useState } from "react";

import logoUrl from "../../assets/screenwide-mark.svg";
import { Alert } from "../../components/base/alert/alert";
import { ScrollArea } from "../../components/base/scroll-area/scroll-area";
import { SidebarNav } from "../../components/base/sidebar-nav/sidebar-nav";
import { Text } from "../../components/base/text/text";
import { WindowHeader } from "../../components/shared/window-header/window-header";
import { WindowShell } from "../../components/shared/window-shell/window-shell";
import { detectedPlatform } from "../../lib/platform";

import { useSettingsApi } from "./settings-api-context";
import { SettingsPanes } from "./settings-panes";
import { sectionTitles, type SettingsSection } from "./settings-sections";
import {
  GeneralSettings,
  GlideSettings,
  RulerSettings,
  OcrSettings,
  ShortcutDefaults,
  ShortcutAction,
  ShortcutSettings,
} from "./types";
import { useGlideSettingsSave } from "./use-glide-settings-save";
import { useOcrSettingsSave } from "./use-ocr-settings-save";
import { useRulerSettingsSave } from "./use-ruler-settings-save";
import { useShortcutCapture } from "./use-shortcut-capture";

const isWindows = detectedPlatform === "windows";

export function SettingsWindow({
  initialSection = "general",
  updateSetting,
}: {
  initialSection?: SettingsSection;
  updateSetting?: ReactNode;
}) {
  const {
    beginShortcutCapture,
    endShortcutCapture,
    getGeneralSettings,
    getGlideSettings,
    getOcrSettings,
    getRulerSettings,
    getShortcutDefaults,
    getShortcutSettings,
    hideSettings,
    minimize,
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

  useEffect(() => {
    Promise.all([
      getGeneralSettings(),
      getGlideSettings(),
      getRulerSettings(),
      getShortcutSettings(),
      getOcrSettings(),
      getShortcutDefaults(),
    ])
      .then(
        ([
          generalSettings,
          glideSettings,
          rulerSettings,
          shortcutSettings,
          ocrSettings,
          shortcutDefaults,
        ]) => {
          setGeneral(generalSettings);
          setGlide(glideSettings);
          setRuler(rulerSettings);
          setSettings(shortcutSettings);
          setOcr(ocrSettings);
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
    getShortcutDefaults,
  ]);

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
          leadingSection={
            <img
              alt="Screenwide"
              className="brightness-0 dark:invert"
              draggable={false}
              src={logoUrl}
            />
          }
          onClose={() => void hideSettings()}
          onMinimize={() => void minimize()}
          // macOS System Settings names the selected pane in the title bar;
          // the Windows Settings app names the app there and heads the page
          // itself, so the section title moves into the content column.
          title={isWindows ? "Settings" : sectionTitles[section]}
        />
      }
    >
      {/* Only the sidebar keeps the window inset; the content column runs to
          the window's right and bottom edges so its scroll shadow lands there,
          and carries the inset on the scrolled content instead. */}
      <div className="gap-layout pl-window-inset flex min-h-0 grow">
        <SidebarNav
          aria-label="Settings sections"
          className="pb-window-inset"
          isExpandable={false}
          isExpanded
          items={[
            { icon: <Settings />, id: "general", label: "General" },
            { icon: <LayoutGrid />, id: "glide", label: "Glide" },
            { icon: <Ruler />, id: "ruler", label: "Ruler" },
            { icon: <ScanText />, id: "ocr", label: "OCR" },
            { icon: <Keyboard />, id: "hotkeys", label: "Shortcuts" },
          ]}
          onSelectionChange={(id) => {
            setSection(id as SettingsSection);
          }}
          selected={section}
        />
        <div className="gap-section flex min-h-0 min-w-0 grow flex-col">
          <ScrollArea
            edgeEffect="shadow"
            key={section}
            rootClassName="min-h-0 min-w-0 grow"
          >
            <section
              aria-label={sectionTitles[section]}
              className="pr-window-inset pb-window-inset flex flex-col"
            >
              {isWindows ? (
                <Text as="h2" className="mb-section" variant="title">
                  {sectionTitles[section]}
                </Text>
              ) : null}
              <SettingsPanes
                defaults={defaults}
                general={general}
                glide={glide}
                ocr={ocr}
                onCaptureChange={onCaptureChange}
                onChangeBinding={changeBinding}
                onChangeGeneral={changeGeneral}
                onChangeGlide={changeGlide}
                onChangeOcr={changeOcr}
                onChangeRuler={(next) => {
                  void changeRuler(next);
                }}
                onError={setError}
                ruler={ruler}
                savingGeneral={savingGeneral}
                savingGlide={savingGlide}
                savingOcr={savingOcr}
                savingRuler={savingRuler}
                savingShortcut={saving}
                section={section}
                shortcuts={settings}
                updateSetting={updateSetting}
              />
            </section>
          </ScrollArea>
          {error ? (
            <Alert
              className="mr-window-inset mb-window-inset"
              color="error"
              role="alert"
            >
              {error}
            </Alert>
          ) : null}
        </div>
      </div>
    </WindowShell>
  );
}
