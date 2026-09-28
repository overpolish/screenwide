// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Text } from "../../../components/base/text/text";
import { AnnotationPixelationGroup } from "../../../components/shared/annotation-style/annotation-pixelation-group";
import { AnnotationRedactionGroup } from "../../../components/shared/annotation-style/annotation-redaction-group";
import { AnnotationWidthSlider } from "../../../components/shared/annotation-style/annotation-width-slider";
import { AnnotationRedaction } from "../../../components/shared/annotation-style/types";
import {
  ANNOTATION_BLUR_STRENGTHS,
  annotationWidthAt,
  annotationWidthIndex,
  redactionSizePresets,
} from "../../../components/shared/annotation-style/widths";

import type { ToolPanelPatch } from "./tool-panel-store";

/**
 * How a redaction covers what is under it: the mode, a pixelation's style, a
 * blur's strength, and a word where the result is only a look. `animated` is
 * whether it draws in over a clip, which shows what it covers until it has
 * arrived.
 */
export function AnnotationRedactionRows({
  animated,
  canShuffle,
  change,
  isLocked,
  pixelation,
  redaction,
  sizes,
  strength,
  width,
}: {
  animated: boolean;
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
  strength: number;
  width: number;
}) {
  // Classic pixelation and blur keep what an attack can read text back out
  // of, and a box drawing in shows what it covers until it has arrived: a
  // look, not a guarantee. Secure pixelation withstands the attack tests.
  const isStylistic =
    redaction === "pixelateClassic" || redaction === "blur" || animated;
  return (
    <>
      <div className="flex items-center justify-between gap-section">
        <span className="text-body text-content-fg">Redaction</span>
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

      {pixelation ? (
        <div className="flex items-center justify-between gap-section">
          <span className="text-body text-content-fg">Style</span>
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
      ) : null}

      {isStylistic ? (
        <Text variant="footnote">
          Use Erase or Colour for sensitive content.
        </Text>
      ) : null}

      {redaction === "blur" ? (
        <div className="flex items-center justify-between gap-section">
          <span className="text-body text-content-fg">Strength</span>
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
      ) : null}
    </>
  );
}
