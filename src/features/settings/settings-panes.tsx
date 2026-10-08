// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type ReactNode } from "react";

import { AnnotateSettingsPanel } from "./sections/annotate-settings";
import { GeneralSettingsPanel } from "./sections/general-settings";
import { GlideSettingsPanel } from "./sections/glide-settings";
import { HotkeySettingsPanel } from "./sections/hotkey-settings";
import { MomentsSettingsPanel } from "./sections/moments-settings";
import { OcrSettingsPanel } from "./sections/ocr-settings";
import { RulerSettingsPanel } from "./sections/ruler-settings";
import { TranscriptionSettingsPanel } from "./sections/transcription-settings";

import type { MomentSettingsState } from "./sections/use-moment-settings";
import type { TranscriptionControls } from "./sections/use-transcription";
import type { SettingsSection } from "./settings-sections";
import type { AnnotateSettings } from "../../bindings/AnnotateSettings";
import type { GeneralSettings } from "../../bindings/GeneralSettings";
import type { GlideSettings } from "../../bindings/GlideSettings";
import type { OcrSettings } from "../../bindings/OcrSettings";
import type { RulerSettings } from "../../bindings/RulerSettings";
import type { ShortcutAction } from "../../bindings/ShortcutAction";
import type { ShortcutDefaults } from "../../bindings/ShortcutDefaults";
import type { ShortcutSettings } from "../../bindings/ShortcutSettings";

export type SettingsPanesProps = {
  annotate: AnnotateSettings | null;
  general: GeneralSettings | null;
  glide: GlideSettings | null;
  moments: MomentSettingsState;
  ocr: OcrSettings | null;
  onCaptureChange: (capturing: boolean) => Promise<void>;
  onChangeAnnotate: (settings: AnnotateSettings) => void;
  onChangeBinding: (action: ShortcutAction, shortcut: string | null) => void;
  onChangeGeneral: (settings: GeneralSettings) => void;
  onChangeGlide: (settings: GlideSettings) => void;
  onChangeOcr: (settings: OcrSettings) => void;
  onChangeRuler: (settings: RulerSettings) => void;
  onError: (message: string) => void;
  ruler: RulerSettings | null;
  savingAnnotate: boolean;
  savingGeneral: boolean;
  savingGlide: boolean;
  savingOcr: boolean;
  savingRuler: boolean;
  savingShortcut: ShortcutAction | null;
  section: SettingsSection;
  shortcuts: ShortcutSettings | null;
  transcription: TranscriptionControls;
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
  annotate,
  defaults,
  general,
  glide,
  moments,
  ocr,
  onCaptureChange,
  onChangeAnnotate,
  onChangeBinding,
  onChangeGeneral,
  onChangeGlide,
  onChangeOcr,
  onChangeRuler,
  onError,
  ruler,
  savingAnnotate,
  savingGeneral,
  savingGlide,
  savingOcr,
  savingRuler,
  savingShortcut,
  section,
  shortcuts,
  transcription,
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
  if (section === "annotate")
    return annotate ? (
      <AnnotateSettingsPanel
        activation={bindingOf(shortcuts, "annotateOverlay") ?? null}
        activationDefault={bindingOf(defaults?.shortcuts, "annotateOverlay")}
        clear={bindingOf(shortcuts, "annotateClear") ?? null}
        clearDefault={bindingOf(defaults?.shortcuts, "annotateClear")}
        isSaving={savingAnnotate || savingShortcut !== null}
        onActivationChange={(value) => {
          onChangeBinding("annotateOverlay", value);
        }}
        onCaptureChange={onCaptureChange}
        onChange={onChangeAnnotate}
        onClearChange={(value) => {
          onChangeBinding("annotateClear", value);
        }}
        settings={annotate}
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
  if (section === "moments")
    return (
      <MomentsSettingsPanel
        moments={moments}
        onCaptureChange={onCaptureChange}
        transcription={transcription}
      />
    );
  if (section === "transcription")
    return <TranscriptionSettingsPanel controls={transcription} />;
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
