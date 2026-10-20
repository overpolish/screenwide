// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AnnotationAngleDial } from "../../../../composites/annotation-style/annotation-angle-dial";
import { ControlRow } from "../../../../composites/control-row/control-row";
import { t } from "../../../../i18n/i18n";
import { RotationRow } from "../rotation-row";

import type { ToolPanelPatch } from "../tool-panel-patch";
import type { ToolPanelAnnotation } from "../tool-panel-selection";

/**
 * How the chosen annotation is turned. A counter is aimed by its tail, on
 * the picture or here: the dial's notch stands where the tail does, and a
 * held Shift snaps it to the same eighth of a turn a drag on the picture
 * snaps to. A placed image turns in depth and flat, as the screen does; a
 * fresh one always lands upright, so there is no turn to choose before one
 * is placed.
 */
export function AnnotationTurnRows({
  annotation,
  change,
  hasAngle,
  isLocked,
}: {
  annotation: ToolPanelAnnotation;
  change: (values: ToolPanelPatch) => void;
  hasAngle: boolean;
  isLocked: boolean;
}) {
  if (annotation.rotation)
    return (
      <RotationRow
        isDisabled={isLocked}
        onChange={(next) => {
          change({ annotationRotation: next });
        }}
        rotation={annotation.rotation}
      />
    );
  if (!hasAngle) return null;
  return (
    <ControlRow title={t("annotation-angle")}>
      {(controlProps) => (
        <div {...controlProps} role="group">
          <AnnotationAngleDial
            isDisabled={isLocked}
            onChange={(next) => {
              change({ annotationAngle: next });
            }}
            value={annotation.angle ?? 0}
          />
        </div>
      )}
    </ControlRow>
  );
}
