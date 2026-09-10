// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type ReactNode } from "react";

import { GeneralSettingsPanel } from "./general-settings";
import { GlideSettingsPanel } from "./glide-settings";
import { HotkeySettingsPanel } from "./hotkey-settings";
import { OcrSettingsPanel } from "./ocr-settings";
import { RulerSettingsPanel } from "./ruler-settings";

import type { SettingsSection } from "./settings-sections";
import type {
  GeneralSettings,
  GlideSettings,
  OcrSettings,
  RulerSettings,
  ShortcutAction,
  ShortcutDefaults,
  ShortcutSettings,
} from "./types";

export type SettingsPanesProps = {
  general: GeneralSettings | null;
  glide: GlideSettings | null;
  ocr: OcrSettings | null;
  onCaptureChange: (capturing: boolean) => Promise<void>;
  onChangeBinding: (action: ShortcutAction, shortcut: string | null) => void;
  onChangeGeneral: (settings: GeneralSettings) => void;
  onChangeGlide: (settings: GlideSettings) => void;
  onChangeOcr: (settings: OcrSettings) => void;
  onChangeRuler: (settings: RulerSettings) => void;
  onError: (message: string) => void;
  ruler: RulerSettings | null;
  savingGeneral: boolean;
  savingGlide: boolean;
  savingOcr: boolean;
  savingRuler: boolean;
  savingShortcut: ShortcutAction | null;
  section: SettingsSection;
  shortcuts: ShortcutSettings | null;
  defaults?: ShortcutDefaults | null;
  updateSetting?: ReactNode;
};

const bindingOf = (
  bindings: ShortcutSettings | null | undefined,
  action: ShortcutAction,
) => bindings?.bindings.find((binding) => binding.action === action)?.shortcut;

/** The pane the sidebar selection asks for, with nothing around it: the
 * window owns the shell and the scrolling. */
export function SettingsPanes({
  defaults,
  general,
  glide,
  ocr,
  onCaptureChange,
  onChangeBinding,
  onChangeGeneral,
  onChangeGlide,
  onChangeOcr,
  onChangeRuler,
  onError,
  ruler,
  savingGeneral,
  savingGlide,
  savingOcr,
  savingRuler,
  savingShortcut,
  section,
  shortcuts,
  updateSetting,
}: SettingsPanesProps) {
  if (section === "general")
    return general ? (
      <GeneralSettingsPanel
        isSaving={savingGeneral}
        onChange={onChangeGeneral}
        onError={onError}
        settings={general}
        updateSetting={updateSetting}
      />
    ) : null;
  if (section === "glide")
    return glide ? (
      <GlideSettingsPanel
        defaults={defaults?.glide}
        isSaving={savingGlide}
        onCaptureChange={onCaptureChange}
        onChange={onChangeGlide}
        settings={glide}
      />
    ) : null;
  if (section === "ruler")
    return ruler ? (
      <RulerSettingsPanel
        activationDefault={bindingOf(defaults?.shortcuts, "rulerOverlay")}
        defaults={defaults?.ruler}
        isSaving={savingRuler}
        onActivationChange={(value) => {
          onChangeBinding("rulerOverlay", value);
        }}
        onCaptureChange={onCaptureChange}
        onChange={onChangeRuler}
        savingShortcut={savingShortcut !== null}
        settings={ruler}
        shortcuts={shortcuts}
      />
    ) : null;
  if (section === "ocr")
    return ocr ? (
      <OcrSettingsPanel
        activation={bindingOf(shortcuts, "recognizeText") ?? null}
        activationDefault={bindingOf(defaults?.shortcuts, "recognizeText")}
        defaults={defaults?.ocr}
        isSaving={savingOcr || savingShortcut !== null}
        onActivationChange={(value) => {
          onChangeBinding("recognizeText", value);
        }}
        onCaptureChange={onCaptureChange}
        onChange={onChangeOcr}
        settings={ocr}
      />
    ) : null;
  return (
    <HotkeySettingsPanel
      defaults={defaults?.shortcuts}
      onCaptureChange={onCaptureChange}
      onChange={onChangeBinding}
      saving={savingShortcut}
      settings={shortcuts}
    />
  );
}
