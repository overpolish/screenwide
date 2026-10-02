// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { PillGroup } from "../../../../components/base/pill-group/pill-group";
import { ControlRow } from "../../../../components/shared/control-row/control-row";
import { CameraOutput } from "../../export/camera-output";

const outputOptions = [
  { id: "combined", label: "One video" },
  { id: "separate", label: "Separate files" },
] satisfies { id: CameraOutput; label: string }[];

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
    <ControlRow title="Output">
      {(controlProps) => (
        <div {...controlProps} role="group">
          <PillGroup
            aria-label="Output"
            display="label"
            isDisabled={isDisabled}
            items={outputOptions}
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
