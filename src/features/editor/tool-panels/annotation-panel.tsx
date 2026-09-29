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
import { ControlRow } from "../../../components/shared/control-row/control-row";
import { ANNOTATION_KINDS } from "../annotation-kinds";
import { EditorKind } from "../types";

import { AnnotationRedactionRows } from "./annotation-redaction-rows";
import { AnnotationSpotlightRows } from "./annotation-spotlight-rows";
import { useAnnotationColorMenu } from "./use-annotation-color-menu";
import { useToolPanelSnapshot } from "./use-tool-panel-snapshot";

/**
 * The chosen annotation's own controls: how big it is drawn, whether it arrives
 * over its clip, and what colour it is - plus the controls its shape has. An
 * arrow carries heads and can be turned round; a counter is a disc with a
 * number in it, which leaves it nothing to reverse and no head to choose; a
 * text box lines its lines up by its alignment; a highlight may be drawn by
 * hand, and drawn again differently; a shape may be drawn by hand too, and
 * rounds its corners; a redaction chooses how it covers and how round its
 * corners are, and offers a pixelation style and a block size only when
 * pixelated, a strength only when blurred and a colour only when filled with
 * one; a spotlight has no colour, only its corners, its fade and its blur.
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
  const showsColor =
    kind.hasColor && (!kind.hasRedaction || redaction === "color");
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
        <ControlRow title="Fit">
          {(controlProps) => (
            <div {...controlProps} role="group">
              <AnnotationFitGroup
                isDisabled={isLocked}
                onChange={(next) => {
                  change({ annotationStyle: { manual: next } });
                }}
                value={manual}
              />
            </div>
          )}
        </ControlRow>
      ) : null}

      {showsSize ? (
        <ControlRow title={kind.sizeLabel}>
          {(controlProps) => (
            <div {...controlProps} role="group">
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
          )}
        </ControlRow>
      ) : null}

      {/* A counter is aimed by its tail, on the picture or here. The dial's
          notch stands where the tail does, and a held Shift snaps it to the
          same eighth of a turn a drag on the picture snaps to. */}
      {kind.hasAngle ? (
        <ControlRow title="Angle">
          {(controlProps) => (
            <div {...controlProps} role="group">
              <AnnotationAngleDial
                isDisabled={isLocked}
                onChange={(next) => {
                  change({ annotationAngle: next });
                }}
                value={annotation.angle ?? 0}
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
          )}
        </ControlRow>
      ) : null}

      {/* Drawing in and out happens over a clip, and only a recording has
          one: a screenshot is one instant, so there is no time for an
          annotation to arrive over and the row is not offered there at all. */}
      {animates ? (
        <ControlRow title="Animate">
          {(controlProps) => (
            <Switch
              {...controlProps}
              isDisabled={isLocked}
              isSelected={annotation.animated}
              onChange={(next) => {
                change({ annotationAnimated: next });
              }}
            />
          )}
        </ControlRow>
      ) : null}

      {kind.hasHead ? (
        <ControlRow title="Head">
          {(controlProps) => (
            <div {...controlProps} role="group">
              <AnnotationHeadGroup
                isDisabled={isLocked}
                onChange={(next) => {
                  change({ annotationStyle: { head: next } });
                }}
                value={head}
              />
            </div>
          )}
        </ControlRow>
      ) : null}

      {/* The head rides the end point, so turning the annotation round points it
          the other way without redrawing it: the same commit path the drag
          on the picture uses. A counter is aimed by its tail instead, on the
          picture itself. */}
      {kind.reversible && !isDraft ? (
        <ControlRow title="Direction">
          {(controlProps) => (
            <div {...controlProps} role="group">
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
                value={align}
              />
            </div>
          )}
        </ControlRow>
      ) : null}

      {kind.hasRadius ? (
        <ControlRow title="Radius">
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
      ) : null}

      <AnnotationSpotlightRows
        change={change}
        hasBlur={kind.hasBlur}
        hasSoftness={kind.hasSoftness}
        isLocked={isLocked}
        style={annotation.style}
      />

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
