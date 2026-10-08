// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { PillGroup } from "../../../../components/base/pill-group/pill-group";
import { ControlRow } from "../../../../components/shared/control-row/control-row";
import { t } from "../../../../i18n/i18n";
import { CameraOutput } from "../../export/camera-output";

const outputOptions = (): { id: CameraOutput; label: string }[] => [
  { id: "combined", label: t("editor-panels-output-combined") },
  { id: "separate", label: t("editor-panels-output-separate") },
];

/**
 * Whether the camera is drawn into the screen's video or saved beside it as a
 * file of its own. It decides what the scenes can place, so it heads the
 * Scene panel, and the workspace follows it: the composed picture, or the
 * screen file alone, the camera saved beside it with only the timeline's
 * cuts.
 */
export function SceneOutputRow({
  cameraOutput,
  isDisabled,
  onChange,
}: {
  cameraOutput: CameraOutput;
  isDisabled: boolean;
  onChange: (output: CameraOutput) => void;
}) {
  return (
    <ControlRow title={t("editor-panels-output")}>
      {(controlProps) => (
        <div {...controlProps} role="group">
          <PillGroup
            aria-label={t("editor-panels-output")}
            display="label"
            isDisabled={isDisabled}
            items={outputOptions()}
            onSelectionChange={(output) => {
              onChange(output as CameraOutput);
            }}
            selected={cameraOutput}
          />
        </div>
      )}
    </ControlRow>
  );
}
