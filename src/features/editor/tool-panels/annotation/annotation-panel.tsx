// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Switch } from "../../../../components/base/switch/switch";
import { Text } from "../../../../components/base/text/text";
import { AnnotationAngleDial } from "../../../../components/shared/annotation-style/annotation-angle-dial";
import { AnnotationColorGrid } from "../../../../components/shared/annotation-style/annotation-color-grid";
import { AnnotationFitGroup } from "../../../../components/shared/annotation-style/annotation-fit-group";
import { AnnotationRadiusField } from "../../../../components/shared/annotation-style/annotation-radius-field";
import { annotationSizeLabel } from "../../../../components/shared/annotation-style/annotation-text";
import { AnnotationWidthSlider } from "../../../../components/shared/annotation-style/annotation-width-slider";
import { redactionSizePresets } from "../../../../components/shared/annotation-style/widths";
import { ControlRow } from "../../../../components/shared/control-row/control-row";
import { t } from "../../../../i18n/i18n";
import { ANNOTATION_KINDS } from "../../annotations/annotation-kinds";
import { EditorKind } from "../../types";
import { useToolPanelSnapshot } from "../use-tool-panel-snapshot";

import { AnnotationActions } from "./annotation-actions";
import { AnnotationOptionRows } from "./annotation-option-rows";
import {
  AnnotationBlurStrengthRow,
  AnnotationRedactionNote,
  AnnotationRedactionRows,
} from "./annotation-redaction-rows";
import { AnnotationSelectionSummary } from "./annotation-selection-summary";
import { ImageFrameRow, ImagePlaybackRow } from "./image-play-rows";
import { ImageSourceRow } from "./image-source-row";
import { ImageSwayRow } from "./image-sway-row";
import { useAnnotationColorMenu } from "./use-annotation-color-menu";

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
 * one; a spotlight has no colour, only its corners, its fade and its blur; a
 * magnifier rounds its zoom area and its loupe and may cast a shadow; a
 * image chooses its picture, its turn and its corners and may cast a
 * shadow; a stroke can clear every stroke off the picture at once.
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
  const {
    annotation,
    annotationColors,
    annotationCount,
    canClearDrawings,
    isLocked,
  } = snapshot;
  const showColorMenu = useAnnotationColorMenu((color) => {
    change({ removeAnnotationColor: color });
  });
  if (annotationCount > 1)
    return (
      <AnnotationSelectionSummary
        count={annotationCount}
        isLocked={isLocked}
        onDelete={() => {
          change({ deleteAnnotations: true });
        }}
      />
    );
  if (!annotation)
    return <Text variant="body">{t("editor-panels-nothing-selected")}</Text>;

  const { color, manual, radius, redaction, strength, width } =
    annotation.style;
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
  const animated = animates && annotation.animated;
  // A placed image whose picture moves; the next one always starts on its
  // first frame and loops.
  const imagePlay = isDraft ? undefined : annotation.image?.play;
  // Rows follow the order every tool panel keeps: mode, size, geometry,
  // options, Animate, colour, notes, then actions. An image's own picture
  // rows - which picture, which frame, how it plays - lead as its mode.
  return (
    <div className="flex flex-col gap-section">
      {annotation.kind === "image" ? (
        <ImageSourceRow
          change={change}
          isLocked={isLocked}
          isPlaced={!isDraft}
        />
      ) : null}
      {imagePlay ? (
        <ImageFrameRow
          change={change}
          isLocked={isLocked}
          play={imagePlay}
          workspace={workspace}
        />
      ) : null}
      {imagePlay && workspace === "recording" ? (
        <ImagePlaybackRow
          change={change}
          isLocked={isLocked}
          play={imagePlay}
        />
      ) : null}

      {kind.hasRedaction ? (
        <AnnotationRedactionRows
          canShuffle={!isDraft}
          change={change}
          isLocked={isLocked}
          pixelation={pixelation}
          redaction={redaction}
          sizes={kind.sizes}
          width={width}
        />
      ) : null}

      {kind.hasFit ? (
        <ControlRow title={t("annotation-fit")}>
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
        <ControlRow title={annotationSizeLabel(kind.sizeMeasure)}>
          {(controlProps) => (
            <div {...controlProps} role="group">
              <AnnotationWidthSlider
                isDisabled={isLocked}
                label={annotationSizeLabel(kind.sizeMeasure)}
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

      {kind.hasRedaction && redaction === "blur" ? (
        <AnnotationBlurStrengthRow
          change={change}
          isLocked={isLocked}
          strength={strength}
        />
      ) : null}

      {kind.hasRadius ? (
        <ControlRow title={t("annotation-radius")}>
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

      {/* A counter is aimed by its tail, and an image turned by its grip, on
          the picture or here. The dial's notch stands where the tail does,
          and a held Shift snaps it to the same eighth of a turn a drag on the
          picture snaps to. A fresh image always stands upright, so there is
          no turn to choose before one is placed. */}
      {kind.hasAngle && !(isDraft && annotation.kind === "image") ? (
        <ControlRow title={t("annotation-angle")}>
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

      <AnnotationOptionRows
        canShuffle={!isDraft}
        change={change}
        isLocked={isLocked}
        kind={kind}
        style={annotation.style}
      />
      <ImageSwayRow
        annotation={annotation}
        change={change}
        isLocked={isLocked}
        workspace={workspace}
      />

      {/* Drawing in and out happens over a clip, and only a recording has
          one: a screenshot is one instant, so the row is not offered there at
          all. It follows every other setting, so its absence moves none. */}
      {animates ? (
        <ControlRow title={t("editor-panels-animate")}>
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

      {kind.hasRedaction ? (
        <AnnotationRedactionNote animated={animated} redaction={redaction} />
      ) : null}

      <AnnotationActions
        canClearDrawings={canClearDrawings}
        change={change}
        isDraft={isDraft}
        isLocked={isLocked}
        kind={annotation.kind}
        reversible={kind.reversible}
      />
    </div>
  );
}
