// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ControlRow } from "../../../composites/control-row/control-row";
import { t } from "../../../i18n/i18n";
import { Badge } from "../../../primitives/badge/badge";
import { formatBytes } from "../../editor/duration";
import { PopupSelect } from "../../popup-panel/popup-select";
import { SettingsGroup } from "../settings-group";

import { useDictationMissingAccess } from "./dictation-access";
import { DictationModelActions } from "./dictation-model-actions";
import {
  transcriptionLanguageName,
  transcriptionLanguages,
} from "./transcription-languages";
import { TranscriptionModelActions } from "./transcription-model-actions";

import type { TranscriptionControls } from "./use-transcription";
import type { TranscriptionModel } from "../../transcription/types";

/** Dictation's row: the model for the language transcription is set to,
 * saying whether words appear as they are spoken, and what holds dictation
 * back while access is not granted. Where it is the Transcripts model, it
 * is removed with Transcripts, not here. */
function DictationRow({
  controls,
  isShared,
  model,
}: {
  controls: TranscriptionControls;
  isShared: boolean;
  model: TranscriptionModel;
}) {
  const needsAccess = useDictationMissingAccess() !== undefined;
  return (
    <ControlRow
      description={
        needsAccess && model.status !== "downloading"
          ? t("settings-transcription-dictation-access")
          : model.description
      }
      title={model.name}
      titleAccessory={<Badge>{formatBytes(model.sizeBytes)}</Badge>}
    >
      {() => (
        <DictationModelActions
          controls={controls}
          isRemovable={!isShared}
          model={model}
        />
      )}
    </ControlRow>
  );
}

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
        {state.models.map((model) =>
          model.purpose === "dictation" ? (
            <DictationRow
              controls={controls}
              isShared={state.models.some(
                (other) =>
                  other.purpose === "transcripts" && other.id === model.id,
              )}
              key={model.purpose}
              model={model}
            />
          ) : (
            <ControlRow
              description={model.description}
              key={model.purpose}
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
          ),
        )}
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
