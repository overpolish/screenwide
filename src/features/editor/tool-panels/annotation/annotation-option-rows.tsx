// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Switch } from "../../../../components/base/switch/switch";
import { AnnotationAlignGroup } from "../../../../components/shared/annotation-style/annotation-align-group";
import { AnnotationHeadGroup } from "../../../../components/shared/annotation-style/annotation-head-group";
import { AnnotationInkGroup } from "../../../../components/shared/annotation-style/annotation-ink-group";
import { AnnotationShuffleSwitch } from "../../../../components/shared/annotation-style/annotation-shuffle-switch";
import { SOFT_SPOTLIGHT_EDGE } from "../../../../components/shared/annotation-style/widths";
import { ControlRow } from "../../../../components/shared/control-row/control-row";
import { t } from "../../../../i18n/i18n";

import type { ANNOTATION_KINDS } from "../../annotations/annotation-kinds";
import type { AnnotationStyle } from "../../annotations/annotations";
import type { ToolPanelPatch } from "../tool-panel-patch";

/**
 * The options only some kinds carry: an arrow's heads, a text box's
 * alignment, a highlight's ink, a hand-drawn stroke, a spotlight's fading
 * edge and blur, and a magnifier's shadow. They sit after the size and
 * geometry rows and before Animate and the colour, the one place every tool
 * panel keeps its options.
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
    | "hasInk"
    | "hasShadow"
    | "hasSoftness"
  >;
  style: AnnotationStyle;
}) {
  return (
    <>
      {kind.hasHead ? (
        <ControlRow title={t("annotation-head")}>
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
        <ControlRow title={t("annotation-alignment")}>
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

      {kind.hasInk ? (
        <ControlRow title={t("annotation-ink")}>
          {(controlProps) => (
            <div {...controlProps} role="group">
              <AnnotationInkGroup
                isDisabled={isLocked}
                onChange={(next) => {
                  change({ annotationStyle: { tint: next } });
                }}
                value={style.tint}
              />
            </div>
          )}
        </ControlRow>
      ) : null}

      {kind.hasHandDrawn ? (
        <ControlRow title={t("annotation-stroke-hand-drawn")}>
          {(controlProps) => (
            <div {...controlProps} role="group">
              <AnnotationShuffleSwitch
                isDisabled={isLocked}
                isSelected={style.handDrawn}
                label={t("annotation-stroke-hand-drawn")}
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
        <ControlRow title={t("editor-panels-soft-edge")}>
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
        <ControlRow title={t("annotation-outside-blur")}>
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
        <ControlRow title={t("editor-panels-drop-shadow")}>
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
