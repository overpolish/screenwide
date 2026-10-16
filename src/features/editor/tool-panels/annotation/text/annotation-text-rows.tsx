// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { TextColorRows } from "../../text-style/text-color-rows";
import { TextEffectsRow } from "../../text-style/text-effects-row";
import { TextFontRow } from "../../text-style/text-font-row";
import { TextShadowRow } from "../../text-style/text-shadow-row";
import { TextTemplateRow } from "../../text-style/text-template-row";

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
 * that changes the rows below it. A choice takes the template's look whole;
 * the template's menu puts it back to how it ships, here and for every text
 * and caption that takes it up after. A text in a screenshot never moves, so
 * its tiles hold on their stills.
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
      onChoose={(annotationLook) => {
        change({ annotationLook });
      }}
      onResetToOriginal={(template) => {
        const shipped = templates.shipped.find(
          (style) => style.template === template,
        );
        change({
          captionDefaults: { original: template },
          ...(shipped ? { annotationLook: shipped } : {}),
        });
      }}
      still={still}
      style={look}
      templates={templates}
    />
  );
}

/**
 * The options a text's look carries beyond its template: its face, which
 * way its shadow falls where it casts one, and its outline where it has no
 * box. The word options follow speech, which a text has none of.
 */
export function AnnotationTextStyleRows({ change, isLocked, look }: RowsProps) {
  const dress = (patch: Partial<TextStyle>) => {
    change({ annotationLook: { ...look, ...patch } });
  };
  return (
    <>
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
        spoken={false}
        style={look}
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
