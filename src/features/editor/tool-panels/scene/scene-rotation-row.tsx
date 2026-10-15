// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { RotateCcw } from "lucide-react";

import { AngleDial } from "../../../../components/base/angle-dial/angle-dial";
import {
  signedAngle,
  wrapAngle,
} from "../../../../components/base/angle-dial/angle-geometry";
import { IconButton } from "../../../../components/base/button/icon-button";
import { ControlRow } from "../../../../components/shared/control-row/control-row";
import { t } from "../../../../i18n/i18n";
import {
  MAX_SCENE_TURN,
  SceneRotation,
} from "../../recording/scenes/recording-scenes";

type Axis = keyof SceneRotation;

/**
 * The turn a dial aimed at `dial` degrees leaves, from `angle`. The dial
 * reads clockwise and Z turns the picture counter-clockwise as it grows, so
 * Z's dial reads it backwards, its notch turning with the picture. The angle
 * moves the short way round to the dial's aim, so a dial swept past twelve
 * o'clock keeps counting rather than jumping a whole turn back, as the
 * scene's turn is blended through every degree between two scenes.
 */
const turnedTo = (axis: Axis, angle: number, dial: number) => {
  const sign = axis === "z" ? -1 : 1;
  const held = sign * angle;
  const next = sign * (held + signedAngle(dial - held));
  return Math.min(MAX_SCENE_TURN, Math.max(-MAX_SCENE_TURN, next));
};

const dialOf = (axis: Axis, angle: number) =>
  wrapAngle(axis === "z" ? -angle : angle);

/**
 * How the scene under the playhead turns the screen in depth, which the
 * select tool's rotation gizmo turns as well: a dial to each axis, its notch
 * in the colour of the axis's ring, and a reset that stands the screen
 * upright, out of reach while it already is. Dials rather than degrees: a
 * turn is judged by eye, and the gizmo is where it is aimed precisely.
 */
export function SceneRotationRow({
  isDisabled,
  onChange,
  rotation,
}: {
  isDisabled: boolean;
  onChange: (rotation: Partial<SceneRotation>) => void;
  rotation: SceneRotation;
}) {
  const axes = [
    { axis: "x", label: t("editor-panels-rotation-x"), tone: "error" },
    { axis: "y", label: t("editor-panels-rotation-y"), tone: "success" },
    { axis: "z", label: t("editor-panels-rotation-z"), tone: "info" },
  ] as const;
  const upright = rotation.x === 0 && rotation.y === 0 && rotation.z === 0;
  return (
    <ControlRow title={t("editor-panels-rotation")}>
      {(controlProps) => (
        <div
          {...controlProps}
          className="flex items-center gap-control"
          role="group"
        >
          <IconButton
            aria-label={t("editor-panels-reset-rotation")}
            isDisabled={isDisabled || upright}
            onPress={() => {
              onChange({ x: 0, y: 0, z: 0 });
            }}
          >
            <RotateCcw />
          </IconButton>
          {axes.map(({ axis, label, tone }) => (
            <AngleDial
              aria-label={label}
              isDisabled={isDisabled}
              key={axis}
              onChange={(dial) => {
                onChange({ [axis]: turnedTo(axis, rotation[axis], dial) });
              }}
              tone={tone}
              value={dialOf(axis, rotation[axis])}
            />
          ))}
        </div>
      )}
    </ControlRow>
  );
}
