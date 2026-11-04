// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ShieldAlert } from "lucide-react";

import { t } from "../../../../../i18n/i18n";
import { Alert } from "../../../../../primitives/alert/alert";
import { IslandParts } from "../../../../../primitives/island/island-parts";
import { IslandRowTitle } from "../../../../../primitives/island/row-title/island-row-title";
import { AnnotationRedactionGroup } from "../../../../annotations/annotation-redaction-group";
import { annotationSizeLabel } from "../../../../annotations/annotation-text";
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
  AnnotationChoiceRow,
  AnnotationColorChoice,
  type AnnotationColorProps,
  AnnotationDelete,
  AnnotationRows,
} from "../annotation-island-parts";
import {
  AnnotationPinNote,
  type AnnotationPinProps,
  AnnotationPinToggle,
} from "../annotation-pin";

import type { AnnotationRedaction } from "../../../../../bindings/AnnotationRedaction";
import type { EditorIslandContent } from "../../editor-island";

// A redaction on the picture, in hand: a box hiding what is under it. Its
// row ends with whether it ramps in, which only a recording has a clip for,
// and deleting it. Under the row, each a row named as the rows round it
// are: how it hides, its style, then its corners, both always there, then
// the one row its style brings last, so its coming and going never moves
// the two: pixelated, its block size; blurred, its strength. A pixelation's
// classic look follows as a toggle on its own line; filled, its colour;
// erased, nothing more. Last, an alert while its settings are not secure: a
// blur or classic pixelation leaves what it covers recoverable, and drawing
// in shows it until the box arrives. It ends the section, so its coming and
// going moves nothing.

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

export type RedactionIslandProps = AnnotationColorProps &
  AnnotationPinProps & {
    /** Whether it can ramp in: a recording's annotations can, a
     * screenshot's, being one instant, cannot. */
    canAnimate: boolean;
    id: string;
    /** Lays a pixelation's blocks out again, or nudges a blur's cells:
     * Pixelate or Blur pressed while it is already the mode. */
    onShuffle: () => void;
    onStyleChange: (style: Partial<RedactionStyle>) => void;
    style: RedactionStyle;
    /** Left out for the pen in hand with nothing chosen. */
    onDelete?: () => void;
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
        <AnnotationPinToggle onPinAction={props.onPinAction} pin={props.pin} />
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
      <IslandParts className="flex flex-col gap-separate">
        {props.pin ? (
          <AnnotationPinNote
            key="pin"
            onPinAction={props.onPinAction}
            pin={props.pin}
          />
        ) : null}
        <AnnotationRows key="rows">
          <AnnotationChoiceRow
            key="style"
            title={t("editor-island-redaction-style")}
          >
            <AnnotationRedactionGroup
              onChange={(next) => {
                onStyleChange({ redaction: next });
              }}
              onShuffle={() => {
                onShuffle();
              }}
              value={redaction}
            />
          </AnnotationChoiceRow>
          <AnnotationChoiceRow key="radius" title={t("annotation-radius")}>
            <CornerChoice
              onChange={(radius) => {
                onStyleChange({ radius });
              }}
              value={style.radius}
            />
          </AnnotationChoiceRow>
          {isPixelated(redaction) ? (
            <AnnotationChoiceRow
              key="block-size"
              title={annotationSizeLabel("blockSize")}
            >
              <BlockSizeChoice
                onChange={(width) => {
                  onStyleChange({ width });
                }}
                sizes={blockSizes}
                value={style.width}
              />
            </AnnotationChoiceRow>
          ) : null}
          {redaction === "blur" ? (
            <AnnotationChoiceRow
              key="strength"
              title={t("annotation-blur-strength")}
            >
              <BlurStrengthChoice
                onChange={(strength) => {
                  onStyleChange({ strength });
                }}
                value={style.strength}
              />
            </AnnotationChoiceRow>
          ) : null}
        </AnnotationRows>
        {isPixelated(redaction) ? (
          <div className="flex" key="classic">
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
          </div>
        ) : null}
        {redaction === "color" ? (
          <AnnotationColorChoice
            color={style.color}
            key="colors"
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
            key="not-secure"
            role="status"
          >
            {t("editor-island-redaction-not-secure")}
          </Alert>
        ) : null}
      </IslandParts>
    ),
    title: (
      <IslandRowTitle
        glyph={<RedactToolIcon aria-hidden className="size-icon" />}
        title={t("editor-island-redaction")}
      />
    ),
  };
}
