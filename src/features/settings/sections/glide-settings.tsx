// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ControlRow } from "../../../composites/control-row/control-row";
import { SliderNumberField } from "../../../composites/slider-number-field/slider-number-field";
import { t } from "../../../i18n/i18n";
import { isMacHost } from "../../../lib/platform";
import { Switch } from "../../../primitives/switch/switch";
import { SettingsGroup } from "../settings-group";
import { PreviewArea } from "../stage/preview-stage";
import { type PreviewFollow } from "../stage/use-preview-subject";

import { glideToggles } from "./glide-rows";

import type { GlideSettings } from "../../../bindings/GlideSettings";

/** What Glide's picture shows: a window moved to the side, or one centred
 * by a double tap. */
export type GlideSubject = "glide" | "glideCenter";

// Glide's held keys live with every other shortcut, in the Shortcuts pane.
// Each row drives the picture: double-tapping to centre shows a window
// centred, every other row Glide moving one to the side.
export function GlideSettingsPanel({
  defaults,
  follow,
  isSaving,
  onChange,
  settings,
}: {
  follow: (subject: GlideSubject) => PreviewFollow;
  isSaving: boolean;
  onChange: (settings: GlideSettings) => void;
  settings: GlideSettings;
  /** What each setting returns to on reset. */
  defaults?: GlideSettings;
}) {
  const update = (changes: Partial<GlideSettings>) => {
    onChange({ ...settings, ...changes });
  };
  const isOff = isSaving || !settings.enabled;
  return (
    <SettingsGroup title={t("settings-section-glide")}>
      <PreviewArea follow={follow("glide")}>
        <ControlRow title={t("settings-glide-enabled")}>
          {(controlProps) => (
            <Switch
              {...controlProps}
              isDisabled={isSaving}
              isSelected={settings.enabled}
              onChange={(enabled) => {
                update({ enabled });
              }}
            />
          )}
        </ControlRow>
      </PreviewArea>
      {glideToggles()
        .filter(({ macOnly }) => !macOnly || isMacHost)
        .map((toggle) => (
          <PreviewArea
            follow={follow(
              toggle.key === "doubleTapCenter" ? "glideCenter" : "glide",
            )}
            key={toggle.key}
          >
            <ControlRow
              description={
                "description" in toggle ? toggle.description : undefined
              }
              title={toggle.title}
            >
              {(controlProps) => (
                <Switch
                  {...controlProps}
                  isDisabled={isOff}
                  isSelected={settings[toggle.key]}
                  onChange={(selected) => {
                    update({ [toggle.key]: selected });
                  }}
                />
              )}
            </ControlRow>
          </PreviewArea>
        ))}
      <PreviewArea follow={follow("glide")}>
        <ControlRow title={t("settings-glide-window-gap")}>
          {(controlProps) => (
            <div {...controlProps} role="group">
              <SliderNumberField
                aria-label={t("settings-glide-window-gap")}
                className="w-56"
                defaultValue={defaults?.windowGap}
                isDisabled={!settings.enabled}
                maxValue={32}
                minValue={0}
                onChange={(windowGap) => {
                  update({ windowGap });
                }}
                unit="px"
                value={settings.windowGap}
              />
            </div>
          )}
        </ControlRow>
      </PreviewArea>
    </SettingsGroup>
  );
}
