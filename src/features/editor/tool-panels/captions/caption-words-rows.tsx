// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Button } from "../../../../components/base/button/button";
import { Switch } from "../../../../components/base/switch/switch";
import { ControlRow } from "../../../../components/shared/control-row/control-row";
import { t } from "../../../../i18n/i18n";

import { CaptionProfanityRows } from "./caption-profanity-rows";

import type { CaptionSettings } from "../../../../bindings/CaptionSettings";

/** The switches that change what the captions say and when, whatever the
 * template, in the panel's order. */
const switches = (): {
  key: keyof Pick<
    CaptionSettings,
    "hideFillers" | "noGaps" | "removePunctuation"
  >;
  title: string;
}[] => [
  { key: "noGaps", title: t("editor-panels-captions-hold-until-next") },
  {
    key: "removePunctuation",
    title: t("editor-panels-captions-hide-punctuation"),
  },
  { key: "hideFillers", title: t("editor-panels-captions-hide-fillers") },
];

/** Whether `settings` say all that `defaults` do. */
const isSame = (settings: CaptionSettings, defaults: CaptionSettings) =>
  (Object.keys(defaults) as (keyof CaptionSettings)[]).every(
    (key) => settings[key] === defaults[key],
  );

/**
 * The Words tab: what the captions say and how long they stay up, whatever
 * the template. Reset puts back the rules last set as default and nothing
 * else: breaks and words placed by hand, and spelling put right in the
 * transcript, are edits of their own and stay. Set as default makes new
 * captions start with these rules.
 */
export function CaptionWordsRows({
  defaults,
  isDisabled,
  onDefaultsChange,
  onSettingsChange,
  settings,
}: {
  defaults: CaptionSettings;
  isDisabled: boolean;
  onDefaultsChange: (settings: CaptionSettings) => void;
  onSettingsChange: (settings: Partial<CaptionSettings>) => void;
  settings: CaptionSettings;
}) {
  const isDefault = isSame(settings, defaults);
  return (
    <>
      {switches().map(({ key, title }) => (
        <ControlRow key={key} title={title}>
          {(controlProps) => (
            <Switch
              {...controlProps}
              isDisabled={isDisabled}
              isSelected={settings[key]}
              onChange={(value) => {
                onSettingsChange({ [key]: value });
              }}
            />
          )}
        </ControlRow>
      ))}

      <CaptionProfanityRows
        isDisabled={isDisabled}
        onChange={onSettingsChange}
        settings={settings}
      />

      <div className="flex flex-wrap justify-end gap-control">
        <Button
          isDisabled={isDisabled || isDefault}
          onPress={() => {
            onSettingsChange(defaults);
          }}
        >
          {t("editor-panels-reset")}
        </Button>
        <Button
          isDisabled={isDisabled || isDefault}
          onPress={() => {
            onDefaultsChange(settings);
          }}
        >
          {t("editor-panels-captions-set-default")}
        </Button>
      </div>
    </>
  );
}
