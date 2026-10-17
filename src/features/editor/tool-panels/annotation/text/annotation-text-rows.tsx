// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { TextColorRows } from "../../text-style/text-color-rows";
import { TextEffectsRow } from "../../text-style/text-effects-row";
import { TextFontRow } from "../../text-style/text-font-row";
import { TextShadowRow } from "../../text-style/text-shadow-row";
import { TextTemplateRow } from "../../text-style/text-template-row";

import { AnnotationTextWordsRow } from "./annotation-text-words-row";
import { asTitleLook, titleTemplates } from "./text-look";
import { setsWordsApart, textWordLines } from "./text-words";

import type { TextSpan } from "../../../../../bindings/TextSpan";
import type { TextStyle } from "../../../../../bindings/TextStyle";
import type { TextTemplates } from "../../../recording/captions/use-text-templates";
import type { ToolPanelPatch } from "../../tool-panel-patch";

type RowsProps = {
  change: (patch: ToolPanelPatch) => void;
  isLocked: boolean;
  look: TextStyle;
};

/**
 * The template a text is dressed in, which leads its panel as the choice
 * that changes the rows below it. A choice takes the template's look whole,
 * its words coming on together; the template's menu puts it back to how it
 * ships, here and for every text and caption that takes it up after. A
 * template that rolls its words through follows speech, so it is not
 * offered. A text in a screenshot never moves, so its tiles hold on their
 * stills.
 */
export function AnnotationTextTemplateRow({
  change,
  isLocked,
  look,
  still,
  templates,
}: RowsProps & { still: boolean; templates: TextTemplates }) {
  return (
    <TextTemplateRow
      isDisabled={isLocked}
      layout={{}}
      onChoose={(chosen) => {
        change({ annotationLook: asTitleLook(chosen) });
      }}
      onResetToOriginal={(template) => {
        const shipped = templates.shipped.find(
          (style) => style.template === template,
        );
        change({
          captionDefaults: { original: template },
          ...(shipped ? { annotationLook: asTitleLook(shipped) } : {}),
        });
      }}
      still={still}
      style={look}
      templates={titleTemplates(templates)}
    />
  );
}

/**
 * The options a text's look carries beyond its template: the words it sets
 * apart, where the look sets any apart and the text has been typed, its
 * face, which way its shadow falls where it casts one, and its effects: its
 * outline where it has no box, and, where it moves, whether its words come
 * on one after another. A text is not said, so nothing follows speech.
 */
export function AnnotationTextStyleRows({
  change,
  isLocked,
  look,
  still,
  words,
}: RowsProps & {
  still: boolean;
  words?: { emphasis?: TextSpan[]; text?: string };
}) {
  const dress = (patch: Partial<TextStyle>) => {
    change({ annotationLook: { ...look, ...patch } });
  };
  const text = words?.text ?? "";
  return (
    <>
      {setsWordsApart(look) && textWordLines(text).length > 0 ? (
        <AnnotationTextWordsRow
          change={change}
          emphasis={words?.emphasis ?? []}
          isLocked={isLocked}
          look={look}
          text={text}
        />
      ) : null}
      <TextFontRow
        font={look.font}
        isDisabled={isLocked}
        onChange={(font) => {
          dress({ font });
        }}
      />
      {look.shadow ? (
        <TextShadowRow
          isDisabled={isLocked}
          onChange={(shadow) => {
            dress({ shadow });
          }}
          shadow={look.shadow}
        />
      ) : null}
      <TextEffectsRow
        isDisabled={isLocked}
        onChange={dress}
        style={look}
        words={still ? [] : ["revealWords"]}
      />
    </>
  );
}

/** A text's colours, as its look offers them. */
export function AnnotationTextColorRows({
  change,
  isLocked,
  look,
  savedColors,
}: RowsProps & { savedColors: string[] }) {
  return (
    <TextColorRows
      isDisabled={isLocked}
      onChange={(patch) => {
        change({ annotationLook: { ...look, ...patch } });
      }}
      savedColors={savedColors}
      style={look}
    />
  );
}
