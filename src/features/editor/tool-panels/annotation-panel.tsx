// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ArrowLeftRight } from "lucide-react";

import { Button } from "../../../components/base/button/button";
import { Switch } from "../../../components/base/switch/switch";
import { Text } from "../../../components/base/text/text";
import { AnnotationAlignGroup } from "../../../components/shared/annotation-style/annotation-align-group";
import { AnnotationAngleDial } from "../../../components/shared/annotation-style/annotation-angle-dial";
import { AnnotationColorGrid } from "../../../components/shared/annotation-style/annotation-color-grid";
import { AnnotationFitGroup } from "../../../components/shared/annotation-style/annotation-fit-group";
import { AnnotationHandDrawnToggle } from "../../../components/shared/annotation-style/annotation-hand-drawn-toggle";
import { AnnotationHeadGroup } from "../../../components/shared/annotation-style/annotation-head-group";
import { AnnotationRadiusField } from "../../../components/shared/annotation-style/annotation-radius-field";
import { AnnotationWidthSlider } from "../../../components/shared/annotation-style/annotation-width-slider";
import { redactionSizePresets } from "../../../components/shared/annotation-style/widths";
import { ANNOTATION_KINDS } from "../annotation-kinds";
import { EditorKind } from "../types";

import { AnnotationRedactionRows } from "./annotation-redaction-rows";
import { useAnnotationColorMenu } from "./use-annotation-color-menu";
import { useToolPanelSnapshot } from "./use-tool-panel-snapshot";

/**
 * The chosen annotation's own controls: how big it is drawn, whether it arrives
 * over its clip, and what colour it is - plus the controls its shape has. An
 * arrow carries heads and can be turned round; a counter is a disc with a
 * number in it, which leaves it nothing to reverse and no head to choose; a
 * text box lines its lines up by its alignment; a highlight may be drawn by
 * hand, and drawn again differently; a redaction chooses how it
 * covers and how round its corners are, and offers a pixelation style and a
 * block size only when pixelated, a strength only when blurred and a colour
 * only when filled with one.
 *
 * This panel belongs to the annotation in hand. It comes up the moment one is
 * chosen, in any tool that can choose one, and every change is committed
 * through the same path the drag on the picture uses, so it lands in the edit
 * history as one edit, and becomes the dress the next annotation is drawn in.
 * With nothing chosen, a drawing tool's panel shows the dress its next
 * annotation will be drawn in, so it is picked before drawing rather than
 * fixed after; what only a drawn annotation can do - turning it round, drawing
 * it again - is not offered there.
 *
 * The head and the colours are the shared controls the live overlay's toolbar
 * carries, so an annotation is dressed the same way wherever it is drawn. A
 * colour the palette does not hold is kept the moment the system panel closes
 * on it, and forgotten from the same right-click menu a saved background is.
 */
export function AnnotationPanel({ workspace }: { workspace: EditorKind }) {
  const { change, snapshot } = useToolPanelSnapshot(workspace);
  const { annotation, annotationColors, isLocked } = snapshot;
  const showColorMenu = useAnnotationColorMenu((color) => {
    change({ removeAnnotationColor: color });
  });
  if (!annotation) return <Text variant="body">Nothing selected</Text>;

  const {
    align,
    color,
    handDrawn,
    head,
    manual,
    radius,
    redaction,
    strength,
    width,
  } = annotation.style;
  const kind = ANNOTATION_KINDS[annotation.kind];
  const isDraft = annotation.isDraft === true;
  const pixelation =
    kind.hasRedaction &&
    (redaction === "pixelate" || redaction === "pixelateClassic")
      ? redaction
      : null;
  const showsSize = kind.hasSize && (!kind.hasRedaction || pixelation !== null);
  const showsColor = !kind.hasRedaction || redaction === "color";
  // A kind that always arrives still has no choice to make before it is drawn.
  const animates =
    workspace === "recording" &&
    kind.animates &&
    !(isDraft && kind.startsStill);
  return (
    <div className="flex flex-col gap-section">
      {kind.hasRedaction ? (
        <AnnotationRedactionRows
          animated={animates && annotation.animated}
          canShuffle={!isDraft}
          change={change}
          isLocked={isLocked}
          pixelation={pixelation}
          redaction={redaction}
          sizes={kind.sizes}
          strength={strength}
          width={width}
        />
      ) : null}

      {kind.hasFit ? (
        <div className="flex items-center justify-between gap-section">
          <span className="text-body text-content-fg">Fit</span>
          <AnnotationFitGroup
            isDisabled={isLocked}
            onChange={(next) => {
              change({ annotationStyle: { manual: next } });
            }}
            value={manual}
          />
        </div>
      ) : null}

      {showsSize ? (
        <div className="flex items-center justify-between gap-section">
          <span className="text-body text-content-fg">{kind.sizeLabel}</span>
          <AnnotationWidthSlider
            isDisabled={isLocked}
            label={kind.sizeLabel}
            onChange={(next) => {
              change({ annotationStyle: { width: next } });
            }}
            presets={redactionSizePresets(redaction, kind.sizes)}
            value={width}
          />
        </div>
      ) : null}

      {/* A counter is aimed by its tail, on the picture or here. The dial's
          notch stands where the tail does, and a held Shift snaps it to the
          same eighth of a turn a drag on the picture snaps to. */}
      {kind.hasAngle ? (
        <div className="flex items-center justify-between gap-section">
          <span className="text-body text-content-fg">Angle</span>
          <AnnotationAngleDial
            isDisabled={isLocked}
            onChange={(next) => {
              change({ annotationAngle: next });
            }}
            value={annotation.angle ?? 0}
          />
        </div>
      ) : null}

      {kind.hasHandDrawn ? (
        <div className="flex items-center justify-between gap-section">
          <span className="text-body text-content-fg">Hand-drawn</span>
          <AnnotationHandDrawnToggle
            isDisabled={isLocked}
            isSelected={handDrawn}
            onChange={(next) => {
              change({ annotationStyle: { handDrawn: next } });
            }}
            onRandomise={
              isDraft
                ? undefined
                : () => {
                    change({ shuffleAnnotation: true });
                  }
            }
          />
        </div>
      ) : null}

      {/* Drawing in and out happens over a clip, and only a recording has
          one: a screenshot is one instant, so there is no time for an
          annotation to arrive over and the row is not offered there at all. */}
      {animates ? (
        <div className="flex items-center justify-between gap-section">
          <span className="text-body text-content-fg">Animate</span>
          <Switch
            aria-label="Animate"
            isDisabled={isLocked}
            isSelected={annotation.animated}
            onChange={(next) => {
              change({ annotationAnimated: next });
            }}
          />
        </div>
      ) : null}

      {kind.hasHead ? (
        <div className="flex items-center justify-between gap-section">
          <span className="text-body text-content-fg">Head</span>
          <AnnotationHeadGroup
            isDisabled={isLocked}
            onChange={(next) => {
              change({ annotationStyle: { head: next } });
            }}
            value={head}
          />
        </div>
      ) : null}

      {/* The head rides the end point, so turning the annotation round points it
          the other way without redrawing it: the same commit path the drag
          on the picture uses. A counter is aimed by its tail instead, on the
          picture itself. */}
      {kind.reversible && !isDraft ? (
        <div className="flex items-center justify-between gap-section">
          <span className="text-body text-content-fg">Direction</span>
          <Button
            aria-label="Reverse the arrow"
            isDisabled={isLocked}
            onPress={() => {
              change({ reverseAnnotation: true });
            }}
          >
            <ArrowLeftRight aria-hidden="true" />
            Reverse
          </Button>
        </div>
      ) : null}

      {kind.hasAlign ? (
        <div className="flex items-center justify-between gap-section">
          <span className="text-body text-content-fg">Alignment</span>
          <AnnotationAlignGroup
            isDisabled={isLocked}
            onChange={(next) => {
              change({ annotationStyle: { align: next } });
            }}
            value={align}
          />
        </div>
      ) : null}

      {kind.hasRedaction ? (
        <div className="flex items-center justify-between gap-section">
          <span className="text-body text-content-fg">Radius</span>
          <AnnotationRadiusField
            isDisabled={isLocked}
            onChange={(next) => {
              change({ annotationStyle: { radius: next } });
            }}
            value={radius}
          />
        </div>
      ) : null}

      {/* The swatches say what they are, so the row carries no heading; it
          keeps the section gap its labelled neighbours sit on. */}
      {showsColor ? (
        <AnnotationColorGrid
          isDisabled={isLocked}
          onChange={(next) => {
            change({ annotationStyle: { color: next } });
          }}
          onContextMenuColor={(kept, anchor) => {
            void showColorMenu(kept, anchor);
          }}
          onSettled={(settled) => {
            change({
              annotationStyle: { color: settled },
              saveAnnotationColor: settled,
            });
          }}
          savedColors={annotationColors}
          value={color}
        />
      ) : null}
    </div>
  );
}
