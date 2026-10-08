// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { RotateCcw } from "lucide-react";

import { Button } from "../../../../components/base/button/button";
import { IconButton } from "../../../../components/base/button/icon-button";
import { Switch } from "../../../../components/base/switch/switch";
import { ControlRow } from "../../../../components/shared/control-row/control-row";
import { SliderNumberField } from "../../../../components/shared/slider-number-field/slider-number-field";
import { t } from "../../../../i18n/i18n";
import { formatDuration } from "../../duration";
import { ToolPanelPatch } from "../tool-panel-patch";
import {
  ToolPanelAudioSelection,
  ToolPanelMicrophone,
} from "../tool-panel-store";

import { ToolProgress } from "./tool-progress";

type RowsProps = {
  change: (values: ToolPanelPatch) => void;
  isLocked: boolean;
};

/**
 * The Select tool's controls for an audio track: how loud it is played back,
 * and for a microphone, the room's noise taken out and the long pauses cut.
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
              rightSection="dB"
              step={1}
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

const silencesDescription = ({
  count,
  durationMs,
  status,
}: ToolPanelMicrophone["silences"]) => {
  if (status === "none-found") return t("editor-panels-silences-none");
  return status === "idle" && count > 0
    ? t("editor-panels-silences-removed", {
        count,
        duration: formatDuration(durationMs),
      })
    : undefined;
};

/**
 * Both tools wait on one listen through the track for speech. Restore all
 * stays in place while there is nothing to restore, as auto zoom's Clear all
 * does, so the row never shifts as cuts come and go. While a tool works on
 * the track for a while, a bar under its row shows how far it has got.
 */
function MicrophoneRows({
  change,
  isLocked,
  microphone,
}: RowsProps & { microphone: ToolPanelMicrophone }) {
  const { noise, noiseProgress, silences } = microphone;
  const isFinding = silences.status === "finding";
  return (
    <>
      <div>
        <ControlRow title={t("editor-panels-reduce-noise")}>
          {(controlProps) => (
            <Switch
              {...controlProps}
              isDisabled={isLocked}
              isSelected={noise === "on" || noise === "cleaning"}
              onChange={(reduceNoise) => {
                change({ reduceNoise });
              }}
            />
          )}
        </ControlRow>
        <ToolProgress
          isWorking={noise === "cleaning"}
          label={t("editor-panels-noise-cleaning")}
          value={noiseProgress}
        />
      </div>
      <div>
        <ControlRow
          controlClassName="gap-control"
          description={silencesDescription(silences)}
          title={t("editor-panels-silences")}
        >
          {(controlProps) => (
            <>
              <Button
                {...controlProps}
                isDisabled={isLocked || isFinding}
                onPress={() => {
                  change({ removeSilences: true });
                }}
              >
                {t("editor-panels-remove")}
              </Button>
              <Button
                aria-label={t("editor-panels-restore-silences")}
                isDisabled={isLocked || isFinding || silences.count === 0}
                onPress={() => {
                  change({ restoreSilences: true });
                }}
              >
                {t("editor-panels-restore-all")}
              </Button>
            </>
          )}
        </ControlRow>
        <ToolProgress
          isWorking={isFinding && silences.progress !== null}
          label={t("editor-panels-silences-finding")}
          value={silences.progress ?? 0}
        />
      </div>
    </>
  );
}
