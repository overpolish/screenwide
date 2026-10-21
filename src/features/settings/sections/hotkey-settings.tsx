// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../i18n/i18n";
import { SettingsGroup } from "../settings-group";

import { GlideKeys } from "./glide-keys";
import {
  type GlobalShortcut,
  recordingShortcuts,
  screenshotShortcuts,
} from "./global-shortcuts";
import { ocrShortcuts } from "./ocr-shortcuts";
import { rulerShortcuts } from "./ruler-shortcuts";
import { ShortcutRow } from "./shortcut-row";

import type { AnnotateSettings } from "../../../bindings/AnnotateSettings";
import type { GlideSettings } from "../../../bindings/GlideSettings";
import type { OcrSettings } from "../../../bindings/OcrSettings";
import type { RulerSettings } from "../../../bindings/RulerSettings";
import type { ShortcutAction } from "../../../bindings/ShortcutAction";
import type { ShortcutDefaults } from "../../../bindings/ShortcutDefaults";
import type { ShortcutSettings } from "../../../bindings/ShortcutSettings";

const bindingOf = (
  bindings: ShortcutSettings | null | undefined,
  action: ShortcutAction,
) => bindings?.bindings.find((binding) => binding.action === action)?.shortcut;

// Every shortcut in one place, grouped by what it works in, in the sidebar's
// order. A feature's group opens with the key that brings it up, then the
// keys that work inside it; with the feature off, its group says so rather
// than greying out unexplained. Each moment's key stays with the moment, as
// part of what a moment is.
export function HotkeySettingsPanel({
  annotate,
  defaults,
  glide,
  ocr,
  onCaptureChange,
  onChange,
  onChangeGlide,
  onChangeOcr,
  onChangeRuler,
  ruler,
  saving,
  savingAnnotate,
  savingGlide,
  savingOcr,
  savingRuler,
  settings,
}: {
  annotate: AnnotateSettings | null;
  glide: GlideSettings | null;
  ocr: OcrSettings | null;
  onCaptureChange: (capturing: boolean) => Promise<void>;
  onChange: (action: ShortcutAction, shortcut: string | null) => void;
  onChangeGlide: (settings: GlideSettings) => void;
  onChangeOcr: (settings: OcrSettings) => void;
  onChangeRuler: (settings: RulerSettings) => void;
  ruler: RulerSettings | null;
  saving: ShortcutAction | null;
  savingAnnotate: boolean;
  savingGlide: boolean;
  savingOcr: boolean;
  savingRuler: boolean;
  settings: ShortcutSettings | null;
  defaults?: ShortcutDefaults | null;
}) {
  const isSavingGlobal = !settings || saving !== null;
  const globalRow = (
    { action, description, label }: GlobalShortcut,
    isOff = false,
  ) => (
    <ShortcutRow
      defaultValue={bindingOf(defaults?.shortcuts, action)}
      description={description}
      isDisabled={isSavingGlobal || isOff}
      key={action}
      label={label}
      onCaptureChange={onCaptureChange}
      onChange={(shortcut) => {
        onChange(action, shortcut);
      }}
      value={bindingOf(settings, action) ?? null}
    />
  );
  const offNote = (isOff: boolean, feature: string) =>
    isOff ? t("settings-shortcuts-feature-off", { feature }) : undefined;

  const glideOff = !glide?.enabled;
  const rulerOff = !ruler?.enabled;
  const annotateOff = !annotate?.enabled;
  const ocrOff = !ocr?.enabled;

  return (
    <div className="flex flex-col gap-group">
      <SettingsGroup title={t("settings-recording")}>
        {recordingShortcuts().map((shortcut) => globalRow(shortcut))}
      </SettingsGroup>
      <SettingsGroup title={t("settings-screenshots")}>
        {screenshotShortcuts().map((shortcut) => globalRow(shortcut))}
      </SettingsGroup>
      {glide ? (
        <SettingsGroup
          description={offNote(glideOff, t("settings-section-glide"))}
          title={t("settings-section-glide")}
        >
          <GlideKeys
            defaults={defaults?.glide}
            isDisabled={glideOff || savingGlide}
            onCaptureChange={onCaptureChange}
            onChange={onChangeGlide}
            settings={glide}
          />
        </SettingsGroup>
      ) : null}
      <SettingsGroup
        description={offNote(rulerOff, t("settings-section-ruler"))}
        title={t("settings-section-ruler")}
      >
        {globalRow(
          { action: "rulerOverlay", label: t("settings-ruler-show") },
          rulerOff,
        )}
        {rulerShortcuts().map(({ action, description, label }) => (
          <ShortcutRow
            captureMode="local-shortcut"
            defaultValue={defaults?.ruler.bindings[action]}
            description={description}
            isDisabled={!ruler || rulerOff || savingRuler}
            key={action}
            label={label}
            onCaptureChange={onCaptureChange}
            onChange={(shortcut) => {
              if (ruler)
                onChangeRuler({
                  ...ruler,
                  bindings: { ...ruler.bindings, [action]: shortcut },
                });
            }}
            value={ruler?.bindings[action] ?? null}
          />
        ))}
      </SettingsGroup>
      <SettingsGroup
        description={offNote(annotateOff, t("settings-section-annotate"))}
        title={t("settings-section-annotate")}
      >
        {globalRow(
          { action: "annotateOverlay", label: t("settings-annotate-draw") },
          annotateOff || savingAnnotate,
        )}
        {globalRow(
          { action: "annotateClear", label: t("settings-annotate-clear") },
          annotateOff || savingAnnotate,
        )}
      </SettingsGroup>
      <SettingsGroup
        description={offNote(ocrOff, t("settings-section-ocr"))}
        title={t("settings-section-ocr")}
      >
        {globalRow(
          { action: "recognizeText", label: t("settings-ocr-activate") },
          ocrOff,
        )}
        {ocrShortcuts().map(({ action, description, label }) => (
          <ShortcutRow
            captureMode="local-shortcut"
            defaultValue={defaults?.ocr.bindings[action]}
            description={description}
            isDisabled={!ocr || ocrOff || savingOcr}
            key={action}
            label={label}
            onCaptureChange={onCaptureChange}
            onChange={(shortcut) => {
              if (ocr)
                onChangeOcr({
                  ...ocr,
                  bindings: { ...ocr.bindings, [action]: shortcut },
                });
            }}
            value={ocr?.bindings[action] ?? null}
          />
        ))}
      </SettingsGroup>
    </div>
  );
}
