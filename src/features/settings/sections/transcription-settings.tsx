// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Badge } from "../../../components/base/badge/badge";
import { GroupBox } from "../../../components/base/group-box/group-box";
import { Text } from "../../../components/base/text/text";
import { ControlRow } from "../../../components/shared/control-row/control-row";
import { t } from "../../../i18n/i18n";
import { formatBytes } from "../../editor/duration";
import { PopupSelect } from "../../popup-panel/popup-select";

import {
  transcriptionLanguageName,
  transcriptionLanguages,
} from "./transcription-languages";
import { TranscriptionModelActions } from "./transcription-model-actions";

import type { TranscriptionControls } from "./use-transcription";

/**
 * The speech models, downloaded to and run on this computer, each named for
 * what it does so nobody has to choose between them, and the language
 * transcription listens for.
 */
export function TranscriptionSettingsPanel({
  controls,
}: {
  controls: TranscriptionControls;
}) {
  const { state } = controls;
  if (!state) return null;
  const languages = [
    {
      id: "system",
      label: t("settings-transcription-system-language", {
        language: transcriptionLanguageName(state.systemLanguage),
      }),
    },
    { id: "auto", label: t("settings-transcription-detect") },
    ...transcriptionLanguages().map((language) => ({
      id: language.code,
      label: language.name,
    })),
  ];
  return (
    <div className="gap-layout flex flex-col">
      {/* First, so it is read before anything is downloaded. */}
      <Text className="px-section" variant="subheadline">
        {t("settings-transcription-local")}
      </Text>
      <GroupBox title={t("settings-transcription-models")}>
        {state.models.map((model) => (
          <ControlRow
            description={model.description}
            key={model.id}
            title={model.name}
            titleAccessory={<Badge>{formatBytes(model.sizeBytes)}</Badge>}
          >
            {() => (
              <TranscriptionModelActions
                controls={controls}
                isRemovable
                model={model}
              />
            )}
          </ControlRow>
        ))}
      </GroupBox>
      <GroupBox title={t("settings-transcription-language")}>
        <ControlRow
          description={t("settings-transcription-language-description")}
          title={t("settings-transcription-language")}
        >
          {(controlProps) => (
            <div className="w-48">
              <PopupSelect
                {...controlProps}
                id="transcription-language"
                items={languages}
                label={t("settings-transcription-language")}
                onSelectionChange={(item) => {
                  controls.setLanguage(item.id);
                }}
                placeholder={t("settings-transcription-language")}
                selectedId={state.language}
              />
            </div>
          )}
        </ControlRow>
      </GroupBox>
    </div>
  );
}
