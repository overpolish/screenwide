// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../../../i18n/i18n";
import { IslandParts } from "../../../../../primitives/island/island-parts";
import { IslandRowTitle } from "../../../../../primitives/island/row-title/island-row-title";
import { PillGroup } from "../../../../../primitives/pill-group/pill-group";
import { alignChoice } from "../../../../annotations/annotation-choices";
import { annotationSizeLabel } from "../../../../annotations/annotation-text";
import { TextToolIcon } from "../../../../annotations/annotation-tool-icons";
import { titleTemplates } from "../../../tools/annotation/text/text-look";
import {
  setsWordsApart,
  textWordLines,
} from "../../../tools/annotation/text/text-words";
import { offersOutline } from "../../../tools/text-style/text-templates";
import {
  AnnotationAnimateToggle,
  AnnotationChoiceRow,
  type AnnotationColorProps,
  AnnotationDelete,
  AnnotationRows,
} from "../annotation-island-parts";
import {
  AnnotationPinNote,
  type AnnotationPinProps,
  AnnotationPinToggle,
} from "../annotation-pin";

import { TextColors } from "./text-colors";
import { TextLookPickers } from "./text-look-pickers";
import { TextSizeChoice } from "./text-size-choice";
import { OutlineToggle, WordByWordToggle } from "./text-toggles";
import { TextWordsChoice } from "./text-words-choice";

import type { AnnotationAlign } from "../../../../../bindings/AnnotationAlign";
import type { FontFamily } from "../../../../../bindings/FontFamily";
import type { TextSpan } from "../../../../../bindings/TextSpan";
import type { TextStyle } from "../../../../../bindings/TextStyle";
import type { TextTemplates } from "../../../recording/captions/use-text-templates";
import type { EditorIslandContent } from "../../editor-island";

// A text placed on the picture, in hand. Its row ends with how it moves,
// which only a recording has a clip for: whether its words come on one after
// another, and whether it grows in; then deleting it. Under the row: its
// template, which decides which lines follow; its size and alignment, each a
// row named as the rows round it are, with its outline after them, a toggle
// on its own line, where it can have one; its faces, each folded to a line
// that opens a column of choices beside the section, and with the accent's
// face the words its template sets apart; then its colours, the one the
// grid sets picked by name where the template uses more than one. Lines
// that come and go with the template sit under the ones that stay, so
// nothing above them moves as templates are tried.

type TextPlacement = {
  align: AnnotationAlign;
  isAnimated: boolean;
  /** The type size in points. */
  size: number;
};

export type TextIslandProps = AnnotationColorProps &
  AnnotationPinProps & {
    /** Whether it can grow in and its words come on one by one: a
     * recording's annotations can, a screenshot's, being one instant,
     * cannot. */
    canAnimate: boolean;
    /** The words set apart, by the bytes they take in the text. */
    emphasis: TextSpan[];
    look: TextStyle;
    onEmphasisChange: (emphasis: TextSpan[]) => void;
    onLookChange: (patch: Partial<TextStyle>) => void;
    onStyleChange: (style: Partial<TextPlacement>) => void;
    style: TextPlacement;
    templates: TextTemplates;
    /** What it says. */
    text: string;
    /** Families beyond the system's, for a story. */
    families?: FontFamily[];
    /** Left out for the pen in hand with nothing chosen. */
    onDelete?: () => void;
    /** An edit made over a stroke, such as words set apart in one drag,
     * ends: everything since its start is one change, to undo as one. */
    onEditGestureEnd?: () => void;
    /** An edit made over a stroke begins. */
    onEditGestureStart?: () => void;
  };

export function textIsland(props: TextIslandProps): EditorIslandContent {
  const { canAnimate, look, onLookChange, onStyleChange, style } = props;
  const setsApart =
    setsWordsApart(look) && textWordLines(props.text).length > 0;
  const hasOutline = offersOutline(look);
  return {
    actions: (
      <>
        <AnnotationPinToggle onPinAction={props.onPinAction} pin={props.pin} />
        {canAnimate ? (
          <>
            <WordByWordToggle
              onChange={(revealWords) => {
                onLookChange({ revealWords });
              }}
              revealWords={look.revealWords}
            />
            <AnnotationAnimateToggle
              isAnimated={style.isAnimated}
              onChange={(isAnimated) => {
                onStyleChange({ isAnimated });
              }}
            />
          </>
        ) : null}
        <AnnotationDelete onDelete={props.onDelete} />
      </>
    ),
    // Hung from the row's start, so the pickers' column opens past its end.
    anchor: "start",
    key: "text",
    section: (
      <TextLookPickers
        families={props.families}
        look={look}
        note={
          props.pin || props.placedInScene ? (
            <AnnotationPinNote
              key="pin"
              onPinAction={props.onPinAction}
              onSceneReset={props.onSceneReset}
              pin={props.pin}
              placedInScene={props.placedInScene}
            />
          ) : null
        }
        onLookChange={onLookChange}
        placement={
          // Outline comes and goes with the template, in place, the rows
          // above it staying where they are.
          <IslandParts className="flex flex-col gap-separate">
            <AnnotationRows key="rows">
              <AnnotationChoiceRow
                key="size"
                title={annotationSizeLabel("size")}
              >
                <TextSizeChoice
                  onChange={(size) => {
                    onStyleChange({ size });
                  }}
                  value={style.size}
                />
              </AnnotationChoiceRow>
              <AnnotationChoiceRow key="align" title={alignChoice.label()}>
                <PillGroup
                  aria-label={alignChoice.label()}
                  items={alignChoice.items()}
                  onSelectionChange={(align) => {
                    onStyleChange({ align: alignChoice.pick(align) });
                  }}
                  selected={alignChoice.select(style.align)}
                />
              </AnnotationChoiceRow>
            </AnnotationRows>
            {hasOutline ? (
              <div className="flex" key="outline">
                <OutlineToggle
                  onChange={(outline) => {
                    onLookChange({ outline });
                  }}
                  outline={look.outline}
                />
              </div>
            ) : null}
          </IslandParts>
        }
        still={!canAnimate}
        templates={titleTemplates(props.templates)}
      >
        {setsApart ? (
          <TextWordsChoice
            emphasis={props.emphasis}
            onChange={props.onEmphasisChange}
            onEditGestureEnd={props.onEditGestureEnd}
            onEditGestureStart={props.onEditGestureStart}
            text={props.text}
          />
        ) : null}
        <TextColors
          look={look}
          onLookChange={onLookChange}
          onRemoveColor={props.onRemoveColor}
          onSaveColor={props.onSaveColor}
          savedColors={props.savedColors}
        />
      </TextLookPickers>
    ),
    title: (
      <IslandRowTitle
        glyph={<TextToolIcon aria-hidden className="size-icon" />}
        title={t("editor-island-text")}
      />
    ),
  };
}
