// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Text } from "../../../../components/base/text/text";
import { AnnotationPixelationGroup } from "../../../../components/shared/annotation-style/annotation-pixelation-group";
import { AnnotationRedactionGroup } from "../../../../components/shared/annotation-style/annotation-redaction-group";
import { AnnotationWidthSlider } from "../../../../components/shared/annotation-style/annotation-width-slider";
import { AnnotationRedaction } from "../../../../components/shared/annotation-style/types";
import {
  ANNOTATION_BLUR_STRENGTHS,
  annotationWidthAt,
  annotationWidthIndex,
  redactionSizePresets,
} from "../../../../components/shared/annotation-style/widths";
import { ControlRow } from "../../../../components/shared/control-row/control-row";

import type { ToolPanelPatch } from "../tool-panel-patch";

/**
 * How a redaction covers what is under it: the mode, and a pixelation's
 * style. They lead the panel because they decide which of its other rows are
 * offered.
 */
export function AnnotationRedactionRows({
  canShuffle,
  change,
  isLocked,
  pixelation,
  redaction,
  sizes,
  width,
}: {
  /** Whether there is a drawn redaction to lay out again. */
  canShuffle: boolean;
  change: (values: ToolPanelPatch) => void;
  isLocked: boolean;
  pixelation: Extract<
    AnnotationRedaction,
    "pixelate" | "pixelateClassic"
  > | null;
  redaction: AnnotationRedaction;
  sizes: number[];
  width: number;
}) {
  return (
    <>
      <ControlRow title="Redaction">
        {(controlProps) => (
          <div {...controlProps} role="group">
            <AnnotationRedactionGroup
              isDisabled={isLocked}
              onChange={(next) => {
                change({ annotationStyle: { redaction: next } });
              }}
              onShuffle={
                canShuffle
                  ? () => {
                      change({ shuffleAnnotation: true });
                    }
                  : undefined
              }
              value={redaction}
            />
          </div>
        )}
      </ControlRow>

      {pixelation ? (
        <ControlRow title="Style">
          {(controlProps) => (
            <div {...controlProps} role="group">
              <AnnotationPixelationGroup
                isDisabled={isLocked}
                onChange={(next) => {
                  // The styles offer different block sizes, so the block moves
                  // to the nearest the new style offers.
                  const presets = redactionSizePresets(next, sizes);
                  change({
                    annotationStyle: {
                      redaction: next,
                      width: annotationWidthAt(
                        annotationWidthIndex(width, presets),
                        presets,
                      ),
                    },
                  });
                }}
                value={pixelation}
              />
            </div>
          )}
        </ControlRow>
      ) : null}
    </>
  );
}

/** A blur's strength, which stands where a pixelation's block size does. */
export function AnnotationBlurStrengthRow({
  change,
  isLocked,
  strength,
}: {
  change: (values: ToolPanelPatch) => void;
  isLocked: boolean;
  strength: number;
}) {
  return (
    <ControlRow title="Strength">
      {(controlProps) => (
        <div {...controlProps} role="group">
          <AnnotationWidthSlider
            isDisabled={isLocked}
            label="Strength"
            onChange={(next) => {
              change({ annotationStyle: { strength: next } });
            }}
            presets={ANNOTATION_BLUR_STRENGTHS}
            value={strength}
          />
        </div>
      )}
    </ControlRow>
  );
}

/**
 * A word where the redaction is only a look. Classic pixelation and blur keep
 * what an attack can read text back out of, and a box drawing in shows what
 * it covers until it has arrived. Secure pixelation withstands the attack
 * tests. The note ends the panel, so its coming and going never moves a
 * control, Animate included.
 */
export function AnnotationRedactionNote({
  animated,
  redaction,
}: {
  /** Whether the redaction draws in over its clip. */
  animated: boolean;
  redaction: AnnotationRedaction;
}) {
  const isStylistic =
    redaction === "pixelateClassic" || redaction === "blur" || animated;
  if (!isStylistic) return null;
  return (
    <Text variant="footnote">Use Erase or Colour for sensitive content.</Text>
  );
}
