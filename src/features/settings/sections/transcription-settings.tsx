// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ControlRow } from "../../../composites/control-row/control-row";
import { t } from "../../../i18n/i18n";
import { Badge } from "../../../primitives/badge/badge";
import { formatBytes } from "../../editor/duration";
import { PopupSelect } from "../../popup-panel/popup-select";
import { SettingsGroup } from "../settings-group";

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
    <div className="flex flex-col gap-group">
      {/* Said first, so it is read before anything is downloaded. */}
      <SettingsGroup description={t("settings-transcription-local")}>
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
      </SettingsGroup>
      <SettingsGroup>
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
      </SettingsGroup>
    </div>
  );
}
