// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useState, useSyncExternalStore } from "react";

import { Button } from "../../../components/base/button/button";
import { GroupBox } from "../../../components/base/group-box/group-box";
import { Switch } from "../../../components/base/switch/switch";
import { Text } from "../../../components/base/text/text";
import { ControlRow } from "../../../components/shared/control-row/control-row";
import { t } from "../../../i18n/i18n";
import { AudioMeter } from "../../audio-inputs/audio-meter";
import { formatBytes } from "../../editor/duration";
import { PopupSelect } from "../../popup-panel/popup-select";
import { useAudioPreview } from "../../recording-inputs/use-audio-preview";
import { useSettingsApi } from "../settings-api-context";

import { TranscriptionModelActions } from "./transcription-model-actions";
import {
  canTranscribe,
  modelFor,
  type TranscriptionControls,
} from "./use-transcription";

import type { MomentSettings } from "../../../bindings/MomentSettings";
import type { PermissionStatus } from "../../permissions/types";
import type { InputDevice } from "../../recording-inputs/types";

const SAME_AS_RECORDING = "same-as-recording";

const watchVisibility = (onChange: () => void) => {
  document.addEventListener("visibilitychange", onChange);
  return () => {
    document.removeEventListener("visibilitychange", onChange);
  };
};
const isPageVisible = () => document.visibilityState === "visible";

/**
 * Whether holding a kind's shortcut records a voice note, and from which
 * microphone, with that microphone's level under its name the way the
 * recording bar shows one. Switching notes on is when microphone access is
 * asked for, so the question never comes up in the middle of a recording.
 */
export function VoiceNoteSettings({
  isSaving,
  onChange,
  settings,
  transcription,
}: {
  isSaving: boolean;
  onChange: (settings: MomentSettings) => void;
  settings: MomentSettings;
  transcription: TranscriptionControls;
}) {
  const momentsModel = modelFor(transcription.state, "moments");
  const isTranscribed = canTranscribe(transcription.state, "moments");
  const {
    getMicrophoneAccess,
    listMicrophones,
    openMicrophoneSettings,
    requestMicrophoneAccess,
  } = useSettingsApi();
  const [access, setAccess] = useState<PermissionStatus | null>(null);
  const [microphones, setMicrophones] = useState<InputDevice[]>([]);

  useEffect(() => {
    void getMicrophoneAccess()
      .then(setAccess)
      .catch(() => undefined);
  }, [getMicrophoneAccess]);

  // Listed only once access is granted: listing devices is what would ask.
  const isGranted = access?.granted === true;
  useEffect(() => {
    if (!isGranted) return;
    void listMicrophones()
      .then(setMicrophones)
      .catch(() => undefined);
  }, [isGranted, listMicrophones]);

  // Settings is hidden rather than closed, so the microphone is let go with
  // the window rather than left open behind it.
  const isVisible = useSyncExternalStore(watchVisibility, isPageVisible);
  const isListening = settings.voiceNotes && isGranted;
  const level = useAudioPreview({
    active: isListening && isVisible,
    deviceId: settings.noteMicrophone ?? undefined,
    kind: "microphone",
  });

  const switchOn = async (voiceNotes: boolean) => {
    if (voiceNotes && access && !access.granted && access.canRequest) {
      await requestMicrophoneAccess().catch(() => undefined);
      setAccess(await getMicrophoneAccess().catch(() => access));
    }
    onChange({ ...settings, voiceNotes });
  };

  const items = [
    { id: SAME_AS_RECORDING, label: t("settings-voice-notes-same-microphone") },
    ...microphones.map((microphone) => ({
      id: microphone.id,
      label: microphone.label,
    })),
  ];
  const isDenied = settings.voiceNotes && access?.granted === false;

  return (
    <GroupBox title={t("settings-voice-notes")}>
      <ControlRow
        description={t("settings-voice-notes-enabled-description")}
        title={t("settings-voice-notes-enabled")}
      >
        {(controlProps) => (
          <Switch
            {...controlProps}
            isDisabled={isSaving}
            isSelected={settings.voiceNotes}
            onChange={(voiceNotes) => {
              void switchOn(voiceNotes);
            }}
          />
        )}
      </ControlRow>
      <ControlRow
        description={t("settings-voice-notes-microphone-description")}
        title={t("settings-voice-notes-microphone")}
      >
        {(controlProps) => (
          <div className="flex w-48 flex-col gap-control-inset">
            <PopupSelect
              {...controlProps}
              id="moments-note-microphone"
              isDisabled={isSaving || !settings.voiceNotes}
              items={items}
              label={t("settings-voice-notes-microphone")}
              onSelectionChange={(item) => {
                onChange({
                  ...settings,
                  noteMicrophone:
                    item.id === SAME_AS_RECORDING ? null : item.id,
                });
              }}
              placeholder={t("settings-voice-notes-microphone")}
              selectedId={settings.noteMicrophone ?? SAME_AS_RECORDING}
            />
            <AudioMeter
              decibels={level.decibels}
              disabled={!isListening}
              height={3}
              hidePeakTick
              hideTicks
              peak={level.peak}
              radius={1.5}
              width="100%"
            />
          </div>
        )}
      </ControlRow>
      <ControlRow
        // The switch says on or off by itself; only a missing model needs
        // a word, to say what the Download beside it fetches.
        description={
          settings.transcribeNotes && !isTranscribed && momentsModel
            ? t("settings-voice-notes-needs-model", {
                model: momentsModel.name,
                size: formatBytes(momentsModel.sizeBytes),
              })
            : undefined
        }
        title={t("settings-voice-notes-transcription")}
      >
        {(controlProps) => (
          <div className="gap-control flex items-center">
            {/* The model is fetched into the one list Settings ›
                Transcription keeps; the switch never fetches or removes it. */}
            {settings.transcribeNotes && !isTranscribed && momentsModel ? (
              <TranscriptionModelActions
                controls={transcription}
                model={momentsModel}
              />
            ) : null}
            <Switch
              {...controlProps}
              isDisabled={isSaving || !settings.voiceNotes}
              isSelected={settings.transcribeNotes}
              onChange={(transcribeNotes) => {
                onChange({ ...settings, transcribeNotes });
              }}
            />
          </div>
        )}
      </ControlRow>
      {isDenied ? (
        <>
          <Text variant="footnote">
            {t("settings-voice-notes-microphone-denied")}
          </Text>
          <div className="gap-control flex justify-end">
            <Button
              onPress={() => {
                void openMicrophoneSettings();
              }}
            >
              {t("permissions-open-settings")}
            </Button>
          </div>
        </>
      ) : null}
    </GroupBox>
  );
}
