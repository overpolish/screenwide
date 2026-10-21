// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ControlRow } from "../../../../composites/control-row/control-row";
import { t } from "../../../../i18n/i18n";
import { Slider } from "../../../../primitives/slider/slider";
import { AnnotationRadiusField } from "../../../annotations/annotation-radius-field";
import { annotationSizeLabel } from "../../../annotations/annotation-text";
import { redactionSizePresets } from "../../../annotations/widths";
import { ANNOTATION_KINDS } from "../../annotations/annotation-kinds";

import type { AnnotationStyle } from "../../annotations/annotations";
import type { ToolPanelPatch } from "../tool-panel-patch";

type Change = (patch: ToolPanelPatch) => void;

/** How large the annotation is drawn, in the measure its kind is sized by,
 * at the steps its kind offers or, for a redaction, its style's. */
export function AnnotationSizeRow({
  change,
  isLocked,
  kind,
  redaction,
  width,
}: {
  change: Change;
  isLocked: boolean;
  kind: (typeof ANNOTATION_KINDS)[keyof typeof ANNOTATION_KINDS];
  redaction: AnnotationStyle["redaction"];
  width: number;
}) {
  return (
    <ControlRow title={annotationSizeLabel(kind.sizeMeasure)}>
      {(controlProps) => (
        <div {...controlProps} role="group">
          <Slider
            aria-label={annotationSizeLabel(kind.sizeMeasure)}
            className="w-48"
            isDisabled={isLocked}
            onChange={(next) => {
              change({ annotationStyle: { width: next } });
            }}
            steps={redactionSizePresets(redaction, kind.sizes)}
            value={width}
          />
        </div>
      )}
    </ControlRow>
  );
}

/** How round a box's corners are. */
export function AnnotationRadiusRow({
  change,
  isLocked,
  radius,
}: {
  change: Change;
  isLocked: boolean;
  radius: number;
}) {
  return (
    <ControlRow title={t("annotation-radius")}>
      {(controlProps) => (
        <div {...controlProps} role="group">
          <AnnotationRadiusField
            isDisabled={isLocked}
            onChange={(next) => {
              change({ annotationStyle: { radius: next } });
            }}
            value={radius}
          />
        </div>
      )}
    </ControlRow>
  );
}
