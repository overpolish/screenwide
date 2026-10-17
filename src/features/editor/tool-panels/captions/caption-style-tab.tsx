// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Button } from "../../../../components/base/button/button";
import { t } from "../../../../i18n/i18n";
import { layoutOf, withLayout } from "../../recording/captions/caption-layout";
import {
  CaptionDefaultsPatch,
  TextTemplates,
} from "../../recording/captions/use-text-templates";
import { SavedColors } from "../text-style/text-color-rows";
import { TextStyleRows } from "../text-style/text-style-rows";
import { TextTemplateRow } from "../text-style/text-template-row";

import type { TextLayout } from "../../../../bindings/TextLayout";
import type { TextStyle } from "../../../../bindings/TextStyle";

/** Whether `a` and `b` hold every option alike, whatever their order. */
const isSameStyle = (a: TextStyle, b: TextStyle) =>
  (Object.keys(a) as (keyof TextStyle)[]).every(
    (key) => JSON.stringify(a[key]) === JSON.stringify(b[key]),
  );

/**
 * The Style tab: the template and the chosen one's look. Choosing a
 * template takes up its look and its layout, but for the layout options
 * changed by hand. Reset and Set as default are about the look alone: Reset
 * brings back the template's look as last set as default, and Set as
 * default makes new captions start with this template looking like this.
 * The template's own menu puts it back to how it ships.
 */
export function CaptionStyleTab({
  isDisabled,
  layout,
  onChange,
  onDefaultsChange,
  savedColors,
  style,
  templates,
}: {
  isDisabled: boolean;
  layout: TextLayout;
  onChange: (style: Partial<TextStyle>) => void;
  onDefaultsChange: (patch: CaptionDefaultsPatch) => void;
  savedColors: SavedColors;
  style: TextStyle;
  templates: TextTemplates;
}) {
  const savedStyle = templates.styles.find(
    (template) => template.template === style.template,
  );
  // The look alone: the layout options the saved look carries are the ones
  // the template ships with, so the captions' own are set aside to compare.
  const look = savedStyle && withLayout(layoutOf(savedStyle), style);
  const isSaved = !!savedStyle && !!look && isSameStyle(look, savedStyle);
  const keepingLayout = (next: TextStyle) => withLayout(layoutOf(style), next);
  const shipped = templates.shipped.find(
    (template) => template.template === style.template,
  );
  return (
    <>
      <TextTemplateRow
        isDisabled={isDisabled}
        layout={layout}
        onChoose={(next) => {
          onChange(withLayout(layout, next));
        }}
        onResetToOriginal={(template) => {
          onDefaultsChange({ original: template });
          // The captions take the original look at once, so the reset
          // shows. It is a look, so the layout stays as it is.
          if (shipped) onChange(keepingLayout(shipped));
        }}
        style={style}
        templates={templates}
      />

      <TextStyleRows
        isDisabled={isDisabled}
        onChange={onChange}
        savedColors={savedColors}
        style={style}
      />

      <div className="flex flex-wrap justify-end gap-control">
        <Button
          isDisabled={isDisabled || !savedStyle || isSaved}
          onPress={() => {
            if (savedStyle) onChange(keepingLayout(savedStyle));
          }}
        >
          {t("editor-panels-reset")}
        </Button>
        <Button
          isDisabled={
            isDisabled ||
            !look ||
            (isSaved && templates.defaultTemplate === style.template)
          }
          onPress={() => {
            if (look) onDefaultsChange({ style: look });
          }}
        >
          {t("editor-panels-captions-set-default")}
        </Button>
      </div>
    </>
  );
}
