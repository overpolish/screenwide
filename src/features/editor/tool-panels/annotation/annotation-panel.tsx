// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ColorGrid } from "../../../../composites/color-grid/color-grid";
import { t } from "../../../../i18n/i18n";
import { ANNOTATION_KINDS } from "../../annotations/annotation-kinds";
import { EditorKind } from "../../types";
import { useToolPanelSnapshot } from "../use-tool-panel-snapshot";

import { AnnotationActions } from "./annotation-actions";
import {
  AnnotationRadiusRow,
  AnnotationSizeRow,
} from "./annotation-geometry-rows";
import {
  AnnotationAnimateRow,
  AnnotationFitRow,
  AnnotationOptionRows,
} from "./annotation-option-rows";
import {
  AnnotationBlurStrengthRow,
  AnnotationRedactionNote,
  AnnotationRedactionRows,
} from "./annotation-redaction-rows";
import { AnnotationSelectionSummary } from "./annotation-selection-summary";
import { AnnotationTurnRows } from "./annotation-turn-rows";
import { ImageFrameRow, ImagePlaybackRow } from "./image-play-rows";
import { ImageSourceRow } from "./image-source-row";
import { ImageSwayRow } from "./image-sway-row";
import {
  AnnotationTextColorRows,
  AnnotationTextStyleRows,
  AnnotationTextTemplateRow,
} from "./text/annotation-text-rows";
import { shownTextLook } from "./text/text-look";

/**
 * The chosen annotation's own controls: how big it is drawn, whether it arrives
 * over its clip, and what colour it is - plus the controls its shape has. An
 * arrow carries heads and can be turned round; a counter is a disc with a
 * number in it, which leaves it nothing to reverse and no head to choose; a
 * text is dressed by a template's look, its face, outline and colours, and
 * lines its lines up by its alignment; a highlight may be drawn by
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
    textTemplates,
  } = snapshot;
  const saveColor = (color: string) => {
    change({ saveAnnotationColor: color });
  };
  const removeColor = (color: string) => {
    change({ removeAnnotationColor: color });
  };
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
    return (
      <p className="text-body text-fg">{t("editor-panels-nothing-selected")}</p>
    );

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
  // A text is dressed by its look, which carries its own colours.
  const look =
    annotation.kind === "text" && textTemplates
      ? shownTextLook(annotation.look, color, textTemplates)
      : null;
  const showsColor =
    kind.hasColor &&
    annotation.kind !== "text" &&
    (!kind.hasRedaction || redaction === "color");
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
      {look && textTemplates ? (
        <AnnotationTextTemplateRow
          change={change}
          isLocked={isLocked}
          look={look}
          still={workspace === "screenshot"}
          templates={textTemplates}
        />
      ) : null}
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
        <AnnotationFitRow change={change} isLocked={isLocked} manual={manual} />
      ) : null}

      {showsSize ? (
        <AnnotationSizeRow
          change={change}
          isLocked={isLocked}
          kind={kind}
          redaction={redaction}
          width={width}
        />
      ) : null}

      {kind.hasRedaction && redaction === "blur" ? (
        <AnnotationBlurStrengthRow
          change={change}
          isLocked={isLocked}
          strength={strength}
        />
      ) : null}

      {/* A focus's strength is the width of its blur, the same five steps a
          blurred redaction offers. */}
      {annotation.kind === "focus" ? (
        <AnnotationBlurStrengthRow
          change={change}
          isLocked={isLocked}
          strength={strength}
        />
      ) : null}

      {kind.hasRadius ? (
        <AnnotationRadiusRow
          change={change}
          isLocked={isLocked}
          radius={radius}
        />
      ) : null}

      <AnnotationTurnRows
        annotation={annotation}
        change={change}
        hasAngle={kind.hasAngle}
        isLocked={isLocked}
      />

      <AnnotationOptionRows
        canShuffle={!isDraft}
        change={change}
        isLocked={isLocked}
        kind={kind}
        style={annotation.style}
      />
      {/* A draft has no words yet to set apart: only a typed text does. */}
      {look ? (
        <AnnotationTextStyleRows
          change={change}
          isLocked={isLocked}
          look={look}
          still={workspace === "screenshot"}
          words={isDraft ? undefined : annotation}
        />
      ) : null}
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
        <AnnotationAnimateRow
          animated={annotation.animated}
          change={change}
          isLocked={isLocked}
        />
      ) : null}
      {/* The swatches say what they are, so the row carries no heading; it
          keeps the section gap its labelled neighbours sit on. */}
      {showsColor ? (
        <ColorGrid
          isDisabled={isLocked}
          offersCustom
          onChange={(next) => {
            change({ annotationStyle: { color: next } });
          }}
          onRemoveColor={removeColor}
          onSaveColor={saveColor}
          savedColors={annotationColors}
          value={color}
        />
      ) : null}

      {look ? (
        <AnnotationTextColorRows
          change={change}
          isLocked={isLocked}
          look={look}
          savedColors={{
            colors: annotationColors,
            onRemove: removeColor,
            onSave: saveColor,
          }}
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
