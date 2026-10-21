// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { RotateCcw } from "lucide-react";

import { ControlRow } from "../../../../composites/control-row/control-row";
import { SliderNumberField } from "../../../../composites/slider-number-field/slider-number-field";
import { t } from "../../../../i18n/i18n";
import { IconButton } from "../../../../primitives/button/icon-button";
import { SilencesRow } from "../silences-row";
import { ToolPanelPatch } from "../tool-panel-patch";
import {
  ToolPanelAudioSelection,
  ToolPanelMicrophone,
} from "../tool-panel-store";

import { ModelSwitchRow, WorkingSwitchRow } from "./microphone-switch-rows";

type RowsProps = {
  change: (values: ToolPanelPatch) => void;
  isLocked: boolean;
};

/**
 * The Select tool's controls for an audio track: how loud it is played back,
 * and for a microphone, the room's noise taken out, the voice cleaned up,
 * its loudness evened out with the system audio making way for it, and the
 * long pauses cut. The volume works on top of Auto volume, so at 0 dB a
 * leveled voice sits at its standard loudness.
 * The track is played at the level it was recorded at until the volume is
 * moved, so its reset is out of reach exactly while it is already there.
 */
export function AudioSelectionRows({
  change,
  isLocked,
  selection,
}: RowsProps & { selection: ToolPanelAudioSelection }) {
  return (
    <div className="flex flex-col gap-section">
      <ControlRow title={t("editor-panels-volume")}>
        {(controlProps) => (
          <div
            {...controlProps}
            className="flex items-center gap-control"
            role="group"
          >
            <IconButton
              aria-label={t("editor-panels-reset-volume")}
              isDisabled={isLocked || selection.decibels === 0}
              onPress={() => {
                change({ audioVolume: 0 });
              }}
            >
              <RotateCcw />
            </IconButton>
            <SliderNumberField
              aria-label={t("editor-panels-volume")}
              className="w-48"
              isDisabled={isLocked}
              maxValue={12}
              minValue={-60}
              onChange={(decibels) => {
                change({ audioVolume: decibels });
              }}
              step={1}
              unit="dB"
              value={selection.decibels}
            />
          </div>
        )}
      </ControlRow>
      {selection.microphone ? (
        <MicrophoneRows
          change={change}
          isLocked={isLocked}
          microphone={selection.microphone}
        />
      ) : null}
    </div>
  );
}

/**
 * Reduce noise and Remove silences wait on one listen through the track for
 * speech. While a tool works on the track for a while, a bar under its row
 * shows how far it has got.
 */
function MicrophoneRows({
  change,
  isLocked,
  microphone,
}: RowsProps & { microphone: ToolPanelMicrophone }) {
  const { ducking, silences } = microphone;
  // Studio sound does Reduce noise's and Vocal cleanup's work while it is on
  // and its model is here, so those two step aside.
  const isStudio =
    microphone.studioModel?.status === "downloaded" &&
    microphone.studio !== "off";
  return (
    <>
      <ModelSwitchRow
        description={t("editor-panels-studio-sound-description")}
        isLocked={isLocked}
        model={microphone.studioModel}
        onChange={(studioSound) => {
          change({ studioSound });
        }}
        onDownload={() => {
          change({ downloadStudioSound: true });
        }}
        progress={microphone.studioProgress}
        state={microphone.studio}
        title={t("editor-panels-studio-sound")}
        working={t("editor-panels-studio-sound-working")}
      />
      {isStudio ? null : (
        <>
          <WorkingSwitchRow
            isLocked={isLocked}
            onChange={(reduceNoise) => {
              change({ reduceNoise });
            }}
            progress={microphone.noiseProgress}
            state={microphone.noise}
            title={t("editor-panels-reduce-noise")}
            working={t("editor-panels-noise-cleaning")}
          />
          <WorkingSwitchRow
            isLocked={isLocked}
            onChange={(vocalCleanup) => {
              change({ vocalCleanup });
            }}
            progress={microphone.voiceProgress}
            state={microphone.voice}
            title={t("editor-panels-vocal-cleanup")}
            working={t("editor-panels-voice-cleaning")}
          />
        </>
      )}
      <WorkingSwitchRow
        isLocked={isLocked}
        onChange={(autoVolume) => {
          change({ autoVolume });
        }}
        progress={microphone.autoVolumeProgress}
        state={microphone.autoVolume}
        title={t("editor-panels-auto-volume")}
        working={t("editor-panels-auto-volume-measuring")}
      />
      {/* Studio ducking changes how the system audio makes way, which it
          does only while Auto volume is on. */}
      {ducking && microphone.autoVolume !== "off" ? (
        <ModelSwitchRow
          description={t("editor-panels-studio-ducking-description")}
          isLocked={isLocked}
          model={ducking.model}
          onChange={(studioDucking) => {
            change({ studioDucking });
          }}
          onDownload={() => {
            change({ downloadStudioDucking: true });
          }}
          progress={ducking.progress}
          state={ducking.state}
          title={t("editor-panels-studio-ducking")}
          working={t("editor-panels-studio-ducking-working")}
        />
      ) : null}
      <SilencesRow
        isLocked={isLocked}
        onRemove={() => {
          change({ removeSilences: true });
        }}
        onRestore={() => {
          change({ restoreSilences: true });
        }}
        silences={silences}
      />
    </>
  );
}
