// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ShieldAlert } from "lucide-react";

import { t } from "../../../../../i18n/i18n";
import { Alert } from "../../../../../primitives/alert/alert";
import { IslandFlow } from "../../../../../primitives/island/island-flow";
import { IslandRowTitle } from "../../../../../primitives/island/row-title/island-row-title";
import { AnnotationRedactionGroup } from "../../../../annotations/annotation-redaction-group";
import { RedactToolIcon } from "../../../../annotations/annotation-tool-icons";
import { BlockSizeChoice } from "../../../../annotations/options/block-size-choice";
import { BlurStrengthChoice } from "../../../../annotations/options/blur-strength-choice";
import { ClassicToggle } from "../../../../annotations/options/classic-toggle";
import { CornerChoice } from "../../../../annotations/options/corner-choice";
import {
  annotationSizes,
  annotationWidthAt,
  annotationWidthIndex,
  redactionSizePresets,
} from "../../../../annotations/widths";
import {
  AnnotationAnimateToggle,
  AnnotationColorChoice,
  type AnnotationColorProps,
  AnnotationDelete,
  AnnotationLine,
} from "../annotation-island-parts";

import type { AnnotationRedaction } from "../../../../../bindings/AnnotationRedaction";
import type { EditorIslandContent } from "../../editor-island";

// A redaction on the picture, in hand: a box hiding what is under it. Its
// row ends with whether it ramps in, which only a recording has a clip for,
// and deleting it. Under the row, how it hides, with its corners: those
// two are always there, so the line that follows, which the mode decides,
// never moves them. Pixelated it offers its style and its block size;
// blurred, its strength; filled, its colour; erased, nothing more. Last, an
// alert while its settings are not secure: a blur or classic pixelation
// leaves what it covers recoverable, and drawing in shows it until the box
// arrives. It ends the section, so its coming and going moves nothing.

type RedactionStyle = {
  /** The fill, while it is filled with a colour. */
  color: string;
  isAnimated: boolean;
  /** How round its corners are, as a percentage of its shorter side. */
  radius: number;
  redaction: AnnotationRedaction;
  /** How wide a blur spreads, 1 the lightest of five. */
  strength: number;
  /** A pixelation's block size in points. */
  width: number;
};

export type RedactionIslandProps = AnnotationColorProps & {
  /** Whether it can ramp in: a recording's annotations can, a
   * screenshot's, being one instant, cannot. */
  canAnimate: boolean;
  id: string;
  onDelete: () => void;
  /** Lays a pixelation's blocks out again, or nudges a blur's cells:
   * Pixelate or Blur pressed while it is already the mode. */
  onShuffle: () => void;
  onStyleChange: (style: Partial<RedactionStyle>) => void;
  style: RedactionStyle;
};

const isPixelated = (
  redaction: AnnotationRedaction,
): redaction is "pixelate" | "pixelateClassic" =>
  redaction === "pixelate" || redaction === "pixelateClassic";

export function redactionIsland(
  props: RedactionIslandProps,
): EditorIslandContent {
  const { canAnimate, id, onDelete, onShuffle, onStyleChange, style } = props;
  const { redaction } = style;
  const blockSizes = redactionSizePresets(redaction, annotationSizes("redact"));
  const isNotSecure =
    redaction === "pixelateClassic" ||
    redaction === "blur" ||
    (canAnimate && style.isAnimated);
  return {
    actions: (
      <>
        {canAnimate ? (
          <AnnotationAnimateToggle
            isAnimated={style.isAnimated}
            onChange={(isAnimated) => {
              onStyleChange({ isAnimated });
            }}
          />
        ) : null}
        <AnnotationDelete onDelete={onDelete} />
      </>
    ),
    anchor: "end",
    key: `redaction-${id}`,
    section: (
      <IslandFlow className="flex flex-col gap-separate">
        <AnnotationLine>
          <AnnotationRedactionGroup
            onChange={(next) => {
              onStyleChange({ redaction: next });
            }}
            onShuffle={() => {
              onShuffle();
            }}
            value={redaction}
          />
          <CornerChoice
            onChange={(radius) => {
              onStyleChange({ radius });
            }}
            value={style.radius}
          />
        </AnnotationLine>
        {isPixelated(redaction) ? (
          <AnnotationLine>
            <ClassicToggle
              isClassic={redaction === "pixelateClassic"}
              onChange={(isClassic) => {
                const next = isClassic ? "pixelateClassic" : "pixelate";
                // The styles offer different block sizes, so the block moves
                // to the nearest step of the new style.
                const presets = redactionSizePresets(
                  next,
                  annotationSizes("redact"),
                );
                onStyleChange({
                  redaction: next,
                  width: annotationWidthAt(
                    annotationWidthIndex(style.width, presets),
                    presets,
                  ),
                });
              }}
            />
            <BlockSizeChoice
              onChange={(width) => {
                onStyleChange({ width });
              }}
              sizes={blockSizes}
              value={style.width}
            />
          </AnnotationLine>
        ) : null}
        {redaction === "blur" ? (
          <AnnotationLine>
            <BlurStrengthChoice
              onChange={(strength) => {
                onStyleChange({ strength });
              }}
              value={style.strength}
            />
          </AnnotationLine>
        ) : null}
        {redaction === "color" ? (
          <AnnotationColorChoice
            color={style.color}
            onChange={(color) => {
              onStyleChange({ color });
            }}
            onRemoveColor={props.onRemoveColor}
            onSaveColor={props.onSaveColor}
            savedColors={props.savedColors}
          />
        ) : null}
        {isNotSecure ? (
          <Alert
            className="px-inset-sm"
            color="error"
            icon={<ShieldAlert aria-hidden />}
            role="status"
          >
            {t("editor-island-redaction-not-secure")}
          </Alert>
        ) : null}
      </IslandFlow>
    ),
    title: (
      <IslandRowTitle
        glyph={<RedactToolIcon aria-hidden className="size-icon" />}
        title={t("editor-island-redaction")}
      />
    ),
  };
}
