// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Switch } from "../../../components/base/switch/switch";
import { SOFT_SPOTLIGHT_EDGE } from "../../../components/shared/annotation-style/widths";
import { ControlRow } from "../../../components/shared/control-row/control-row";

import type { AnnotationStyle } from "../annotations";
import type { ToolPanelPatch } from "./tool-panel-store";

/**
 * A spotlight's own controls: whether its edge fades in from its box, and
 * whether what lies outside it is blurred as well as dimmed. How dark the dim
 * is, is not a choice.
 */
export function AnnotationSpotlightRows({
  change,
  hasBlur,
  hasSoftness,
  isLocked,
  style,
}: {
  change: (values: ToolPanelPatch) => void;
  hasBlur: boolean;
  hasSoftness: boolean;
  isLocked: boolean;
  style: AnnotationStyle;
}) {
  return (
    <>
      {hasSoftness ? (
        <ControlRow title="Soft edge">
          {(controlProps) => (
            <Switch
              {...controlProps}
              isDisabled={isLocked}
              isSelected={style.softness > 0}
              onChange={(next) => {
                change({
                  annotationStyle: { softness: next ? SOFT_SPOTLIGHT_EDGE : 0 },
                });
              }}
            />
          )}
        </ControlRow>
      ) : null}

      {hasBlur ? (
        <ControlRow title="Blur">
          {(controlProps) => (
            <Switch
              {...controlProps}
              isDisabled={isLocked}
              isSelected={style.blur}
              onChange={(next) => {
                change({ annotationStyle: { blur: next } });
              }}
            />
          )}
        </ControlRow>
      ) : null}
    </>
  );
}
