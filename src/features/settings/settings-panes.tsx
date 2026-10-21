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
  // Each tool is a box of its own: what it does, then how it behaves; their
  // keys are with every other shortcut.
  if (section === "tools")
    return (
      <div className="flex flex-col gap-group">
        {glide ? (
          <GlideSettingsPanel
            defaults={defaults?.glide}
            isSaving={savingGlide}
            onChange={onChangeGlide}
            settings={glide}
          />
        ) : null}
        {ruler ? (
          <RulerSettingsPanel
            isSaving={savingRuler}
            onChange={onChangeRuler}
            settings={ruler}
          />
        ) : null}
        {annotate ? (
          <AnnotateSettingsPanel
            isSaving={savingAnnotate}
            onChange={onChangeAnnotate}
            settings={annotate}
          />
        ) : null}
        {ocr ? (
          <OcrSettingsPanel
            isSaving={savingOcr}
            onChange={onChangeOcr}
            settings={ocr}
          />
        ) : null}
      </div>
    );
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
      annotate={annotate}
      defaults={defaults}
      glide={glide}
      ocr={ocr}
      onCaptureChange={onCaptureChange}
      onChange={onChangeBinding}
      onChangeGlide={onChangeGlide}
      onChangeOcr={onChangeOcr}
      onChangeRuler={onChangeRuler}
      ruler={ruler}
      saving={savingShortcut}
      savingAnnotate={savingAnnotate}
      savingGlide={savingGlide}
      savingOcr={savingOcr}
      savingRuler={savingRuler}
      settings={shortcuts}
    />
  );
}
