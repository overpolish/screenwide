// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Badge } from "../../../components/base/badge/badge";
import { GroupBox } from "../../../components/base/group-box/group-box";
import { Text } from "../../../components/base/text/text";
import { ControlRow } from "../../../components/shared/control-row/control-row";
import { formatBytes } from "../../editor/duration";
import { PopupSelect } from "../../popup-panel/popup-select";

import {
  TRANSCRIPTION_LANGUAGES,
  transcriptionLanguageName,
} from "./transcription-languages";
import { TranscriptionModelActions } from "./transcription-model-actions";

import type { TranscriptionControls } from "./use-transcription";

/**
 * The speech-to-text models, downloaded to and run on this computer, each
 * named for what it transcribes so nobody has to choose between them, and
 * the language they listen for.
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
      label: `System (${transcriptionLanguageName(state.systemLanguage)})`,
    },
    { id: "auto", label: "Detect Automatically" },
    ...TRANSCRIPTION_LANGUAGES.map((language) => ({
      id: language.code,
      label: language.name,
    })),
  ];
  return (
    <div className="gap-layout flex flex-col">
      {/* First, so it is read before anything is downloaded. */}
      <Text className="px-section" variant="subheadline">
        Transcription runs on this computer. Audio is never uploaded.
      </Text>
      <GroupBox title="Models">
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
      <GroupBox title="Language">
        <ControlRow
          description="The language your recordings are spoken in."
          title="Language"
        >
          {(controlProps) => (
            <div className="w-48">
              <PopupSelect
                {...controlProps}
                id="transcription-language"
                items={languages}
                label="Language"
                onSelectionChange={(item) => {
                  controls.setLanguage(item.id);
                }}
                placeholder="Language"
                selectedId={state.language}
              />
            </div>
          )}
        </ControlRow>
      </GroupBox>
    </div>
  );
}
