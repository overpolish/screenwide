// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../../../i18n/i18n";
import { IslandRowTitle } from "../../../../../primitives/island/row-title/island-row-title";
import { PillGroup } from "../../../../../primitives/pill-group/pill-group";
import { alignChoice } from "../../../../annotations/annotation-choices";
import { TextToolIcon } from "../../../../annotations/annotation-tool-icons";
import {
  setsWordsApart,
  textWordLines,
} from "../../../tool-panels/annotation/text/text-words";
import { offersOutline } from "../../../tool-panels/text-style/text-templates";
import {
  AnnotationAnimateToggle,
  type AnnotationColorProps,
  AnnotationDelete,
  AnnotationLine,
} from "../annotation-island-parts";

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
// template, which decides which lines follow; its size and alignment, with
// its outline where it can have one; its faces, each folded to a line that
// opens a column of choices beside the section, and with the accent's face
// the words its template sets apart; then its colours, the one the grid sets
// picked by name where the template uses more than one. Lines that come and
// go with the template sit under the ones that stay, so nothing above them
// moves as templates are tried.

type TextPlacement = {
  align: AnnotationAlign;
  isAnimated: boolean;
  /** The type size in points. */
  size: number;
};

export type TextIslandProps = AnnotationColorProps & {
  /** Whether it can grow in and its words come on one by one: a
   * recording's annotations can, a screenshot's, being one instant,
   * cannot. */
  canAnimate: boolean;
  /** The words set apart, by the bytes they take in the text. */
  emphasis: TextSpan[];
  id: string;
  look: TextStyle;
  onDelete: () => void;
  onEmphasisChange: (emphasis: TextSpan[]) => void;
  onLookChange: (patch: Partial<TextStyle>) => void;
  onStyleChange: (style: Partial<TextPlacement>) => void;
  style: TextPlacement;
  templates: TextTemplates;
  /** What it says. */
  text: string;
  /** Families beyond the system's, for a story. */
  families?: FontFamily[];
  /** An edit made over a stroke, such as words set apart in one drag,
   * ends: everything since its start is one change, to undo as one. */
  onEditGestureEnd?: () => void;
  /** An edit made over a stroke begins. */
  onEditGestureStart?: () => void;
};

export function textIsland(props: TextIslandProps): EditorIslandContent {
  const { canAnimate, id, look, onLookChange, onStyleChange, style } = props;
  const setsApart =
    setsWordsApart(look) && textWordLines(props.text).length > 0;
  const hasOutline = offersOutline(look);
  return {
    actions: (
      <>
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
    key: `text-${id}`,
    section: (
      <TextLookPickers
        families={props.families}
        look={look}
        onLookChange={onLookChange}
        placement={
          <AnnotationLine>
            <TextSizeChoice
              onChange={(size) => {
                onStyleChange({ size });
              }}
              value={style.size}
            />
            <PillGroup
              aria-label={alignChoice.label()}
              items={alignChoice.items()}
              onSelectionChange={(align) => {
                onStyleChange({ align: alignChoice.pick(align) });
              }}
              selected={alignChoice.select(style.align)}
            />
            {hasOutline ? (
              <OutlineToggle
                onChange={(outline) => {
                  onLookChange({ outline });
                }}
                outline={look.outline}
              />
            ) : null}
          </AnnotationLine>
        }
        still={!canAnimate}
        templates={props.templates}
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
