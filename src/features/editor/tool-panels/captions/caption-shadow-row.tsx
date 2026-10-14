// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AnnotationAngleDial } from "../../../../components/shared/annotation-style/annotation-angle-dial";
import { ControlRow } from "../../../../components/shared/control-row/control-row";
import { t } from "../../../../i18n/i18n";

import type { CaptionShadow } from "../../../../bindings/CaptionShadow";

/** Between the dial's twelve o'clock, which a caption's shadow measures
 * from, and east, which the annotations' dial is given angles from. */
const QUARTER_TURN = 90;

/**
 * Which way the words' shadow falls, where the template casts one: the
 * annotations' dial, its notch where the shadow falls. How far it falls and
 * how soft it is are the template's.
 */
export function CaptionShadowRow({
  isDisabled,
  onChange,
  shadow,
}: {
  isDisabled: boolean;
  onChange: (shadow: CaptionShadow) => void;
  shadow: CaptionShadow;
}) {
  return (
    <ControlRow title={t("editor-panels-captions-shadow")}>
      {(controlProps) => (
        <div {...controlProps} role="group">
          <AnnotationAngleDial
            isDisabled={isDisabled}
            onChange={(radians) => {
              const degrees = (radians * 180) / Math.PI + QUARTER_TURN;
              onChange({ ...shadow, angle: ((degrees % 360) + 360) % 360 });
            }}
            value={((shadow.angle - QUARTER_TURN) * Math.PI) / 180}
          />
        </div>
      )}
    </ControlRow>
  );
}
