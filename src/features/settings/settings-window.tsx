// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Keyboard, LayoutGrid, Ruler, ScanText, Settings } from "lucide-react";
import { type ReactNode, useCallback, useEffect, useState } from "react";

import logoUrl from "../../assets/screenwide-mark.svg";
import { ScrollArea } from "../../components/base/scroll-area/scroll-area";
import { SidebarNav } from "../../components/base/sidebar-nav/sidebar-nav";
import { Text } from "../../components/base/text/text";
import { WindowHeader } from "../../components/shared/window-header/window-header";

import { GeneralSettingsPanel } from "./general-settings";
import { GlideSettingsPanel } from "./glide-settings";
import { HotkeySettingsPanel } from "./hotkey-settings";
import { OcrSettingsPanel } from "./ocr-settings";
import { RulerSettingsPanel } from "./ruler-settings";
import { useSettingsApi } from "./settings-api-context";
import { LiveSettingsUpdateActions } from "./settings-update-actions";
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

type SettingsSection = "general" | "glide" | "ruler" | "ocr" | "hotkeys";

const sectionTitles: Record<SettingsSection, string> = {
  general: "General",
  glide: "Glide",
  hotkeys: "Shortcuts",
  ocr: "OCR",
  ruler: "Ruler",
};

export function SettingsWindow({
  initialSection = "general",
  updateActions = <LiveSettingsUpdateActions />,
}: {
  initialSection?: SettingsSection;
  updateActions?: ReactNode;
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
    <main className="window-surface gap-section flex h-full w-full flex-col overflow-hidden rounded-window text-content-fg">
      <WindowHeader
        actions={updateActions}
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
        title="Settings"
      />
      <div className="gap-layout px-window-inset pb-window-inset flex min-h-0 grow">
        <SidebarNav
          aria-label="Settings sections"
          isExpandable={false}
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
          <header className="w-full shrink-0">
            <h1 className="m-0 text-lg font-semibold">
              {sectionTitles[section]}
            </h1>
          </header>
          <section
            aria-label={sectionTitles[section]}
            className="gap-section flex min-h-0 min-w-0 grow flex-col"
          >
            <ScrollArea
              edgeEffect="inset"
              key={section}
              rootClassName="min-h-0 grow"
              scrollbarAutoHide="never"
            >
              {section === "general" && general ? (
                <GeneralSettingsPanel
                  isSaving={savingGeneral}
                  onChange={changeGeneral}
                  onError={setError}
                  settings={general}
                />
              ) : null}
              {section === "glide" && glide ? (
                <GlideSettingsPanel
                  defaults={defaults?.glide}
                  isSaving={savingGlide}
                  onCaptureChange={onCaptureChange}
                  onChange={changeGlide}
                  settings={glide}
                />
              ) : null}
              {section === "ruler" && ruler ? (
                <RulerSettingsPanel
                  activationDefault={
                    defaults?.shortcuts.bindings.find(
                      (binding) => binding.action === "rulerOverlay",
                    )?.shortcut
                  }
                  defaults={defaults?.ruler}
                  isSaving={savingRuler}
                  onActivationChange={(value) => {
                    changeBinding("rulerOverlay", value);
                  }}
                  onCaptureChange={onCaptureChange}
                  onChange={(next) => {
                    void changeRuler(next);
                  }}
                  savingShortcut={saving !== null}
                  settings={ruler}
                  shortcuts={settings}
                />
              ) : null}
              {section === "ocr" && ocr ? (
                <OcrSettingsPanel
                  activation={
                    settings?.bindings.find(
                      (binding) => binding.action === "recognizeText",
                    )?.shortcut ?? null
                  }
                  activationDefault={
                    defaults?.shortcuts.bindings.find(
                      (binding) => binding.action === "recognizeText",
                    )?.shortcut
                  }
                  defaults={defaults?.ocr}
                  isSaving={savingOcr || saving !== null}
                  onActivationChange={(value) => {
                    changeBinding("recognizeText", value);
                  }}
                  onCaptureChange={onCaptureChange}
                  onChange={changeOcr}
                  settings={ocr}
                />
              ) : null}
              {section === "hotkeys" ? (
                <HotkeySettingsPanel
                  defaults={defaults?.shortcuts}
                  onCaptureChange={onCaptureChange}
                  onChange={changeBinding}
                  saving={saving}
                  settings={settings}
                />
              ) : null}
            </ScrollArea>
            {error ? (
              <Text className="text-error" role="alert" variant="help">
                {error}
              </Text>
            ) : null}
          </section>
        </div>
      </div>
    </main>
  );
}
