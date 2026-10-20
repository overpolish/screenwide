// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ControlRow } from "../../../../composites/control-row/control-row";
import { t } from "../../../../i18n/i18n";
import { PillGroup } from "../../../../primitives/pill-group/pill-group";

import type { CaptionProfanity } from "../../../../bindings/CaptionProfanity";
import type { CaptionSettings } from "../../../../bindings/CaptionSettings";

/** The pill for profanity shown as it was said, which the settings keep as
 * profanity not hidden at all. */
const AS_SAID = "said";

const profanityOptions = (): {
  id: CaptionProfanity | typeof AS_SAID;
  label: string;
}[] => [
  { id: AS_SAID, label: t("editor-panels-captions-profanity-said") },
  { id: "mask", label: t("editor-panels-captions-profanity-partly") },
  { id: "hide", label: t("editor-panels-captions-profanity-fully") },
  { id: "remove", label: t("editor-panels-captions-profanity-hidden") },
];

/** How the captions show profanity: as it was said, partly or fully starred
 * out, or left out. One choice stands for the settings' switch and style. */
export function CaptionProfanityRows({
  isDisabled,
  onChange,
  settings,
}: {
  isDisabled: boolean;
  onChange: (settings: Partial<CaptionSettings>) => void;
  settings: CaptionSettings;
}) {
  return (
    <ControlRow title={t("editor-panels-captions-profanity-style")}>
      {(controlProps) => (
        <div {...controlProps} role="group">
          <PillGroup
            aria-label={t("editor-panels-captions-profanity-style")}
            display="label"
            isDisabled={isDisabled}
            items={profanityOptions()}
            onSelectionChange={(choice) => {
              onChange(
                choice === AS_SAID
                  ? { censorProfanity: false }
                  : {
                      censorProfanity: true,
                      profanity: choice as CaptionProfanity,
                    },
              );
            }}
            selected={settings.censorProfanity ? settings.profanity : AS_SAID}
          />
        </div>
      )}
    </ControlRow>
  );
}
