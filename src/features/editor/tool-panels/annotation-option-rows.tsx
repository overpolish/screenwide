// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Switch } from "../../../components/base/switch/switch";
import { AnnotationAlignGroup } from "../../../components/shared/annotation-style/annotation-align-group";
import { AnnotationHandDrawnToggle } from "../../../components/shared/annotation-style/annotation-hand-drawn-toggle";
import { AnnotationHeadGroup } from "../../../components/shared/annotation-style/annotation-head-group";
import { SOFT_SPOTLIGHT_EDGE } from "../../../components/shared/annotation-style/widths";
import { ControlRow } from "../../../components/shared/control-row/control-row";

import type { ANNOTATION_KINDS } from "../annotation-kinds";
import type { AnnotationStyle } from "../annotations";
import type { ToolPanelPatch } from "./tool-panel-store";

/**
 * The options only some kinds carry: an arrow's heads, a text box's
 * alignment, a hand-drawn stroke, a spotlight's fading edge and blur, and a
 * magnifier's shadow. They sit after the size and geometry rows and
 * before Animate and the colour, the one place every tool panel keeps its
 * options.
 */
export function AnnotationOptionRows({
  canShuffle,
  change,
  isLocked,
  kind,
  style,
}: {
  /** Whether there is a drawn annotation to draw again differently. */
  canShuffle: boolean;
  change: (values: ToolPanelPatch) => void;
  isLocked: boolean;
  kind: Pick<
    (typeof ANNOTATION_KINDS)[keyof typeof ANNOTATION_KINDS],
    | "hasAlign"
    | "hasBlur"
    | "hasHandDrawn"
    | "hasHead"
    | "hasShadow"
    | "hasSoftness"
  >;
  style: AnnotationStyle;
}) {
  return (
    <>
      {kind.hasHead ? (
        <ControlRow title="Head">
          {(controlProps) => (
            <div {...controlProps} role="group">
              <AnnotationHeadGroup
                isDisabled={isLocked}
                onChange={(next) => {
                  change({ annotationStyle: { head: next } });
                }}
                value={style.head}
              />
            </div>
          )}
        </ControlRow>
      ) : null}

      {kind.hasAlign ? (
        <ControlRow title="Alignment">
          {(controlProps) => (
            <div {...controlProps} role="group">
              <AnnotationAlignGroup
                isDisabled={isLocked}
                onChange={(next) => {
                  change({ annotationStyle: { align: next } });
                }}
                value={style.align}
              />
            </div>
          )}
        </ControlRow>
      ) : null}

      {kind.hasHandDrawn ? (
        <ControlRow title="Hand-drawn">
          {(controlProps) => (
            <div {...controlProps} role="group">
              <AnnotationHandDrawnToggle
                isDisabled={isLocked}
                isSelected={style.handDrawn}
                onChange={(next) => {
                  change({ annotationStyle: { handDrawn: next } });
                }}
                onRandomise={
                  canShuffle
                    ? () => {
                        change({ shuffleAnnotation: true });
                      }
                    : undefined
                }
              />
            </div>
          )}
        </ControlRow>
      ) : null}

      {kind.hasSoftness ? (
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

      {kind.hasBlur ? (
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

      {kind.hasShadow ? (
        <ControlRow title="Drop shadow">
          {(controlProps) => (
            <Switch
              {...controlProps}
              isDisabled={isLocked}
              isSelected={style.shadow}
              onChange={(next) => {
                change({ annotationStyle: { shadow: next } });
              }}
            />
          )}
        </ControlRow>
      ) : null}
    </>
  );
}
