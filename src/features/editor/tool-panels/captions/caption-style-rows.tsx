// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Switch } from "../../../../components/base/switch/switch";
import { ControlRow } from "../../../../components/shared/control-row/control-row";
import { t } from "../../../../i18n/i18n";

import { CaptionColorRows } from "./caption-color-rows";
import { CaptionFontRow } from "./caption-font-row";
import { offersOutline, offersWordHighlight } from "./caption-templates";

import type { CaptionStyle } from "../../../../bindings/CaptionStyle";

/**
 * A caption template's look, which it sets and saves as its default: the
 * face, whether the word being said stands out, the outline, and the
 * colours. How large, where and how many words at a time are the Layout
 * tab's.
 */
export function CaptionStyleRows({
  isDisabled,
  onChange,
  savedColors,
  style,
}: {
  isDisabled: boolean;
  onChange: (style: Partial<CaptionStyle>) => void;
  savedColors: string[];
  style: CaptionStyle;
}) {
  return (
    <>
      <CaptionFontRow
        font={style.font}
        isDisabled={isDisabled}
        onChange={(font) => {
          onChange({ font });
        }}
      />

      {offersWordHighlight(style) ? (
        <ControlRow title={t("editor-panels-captions-highlight-word")}>
          {(controlProps) => (
            <Switch
              {...controlProps}
              isDisabled={isDisabled}
              isSelected={style.highlightWords}
              onChange={(highlightWords) => {
                onChange({ highlightWords });
              }}
            />
          )}
        </ControlRow>
      ) : null}

      {offersOutline(style) ? (
        <ControlRow title={t("editor-panels-captions-outline")}>
          {(controlProps) => (
            <Switch
              {...controlProps}
              isDisabled={isDisabled}
              isSelected={style.outline}
              onChange={(outline) => {
                onChange({ outline });
              }}
            />
          )}
        </ControlRow>
      ) : null}

      <CaptionColorRows
        isDisabled={isDisabled}
        onChange={onChange}
        savedColors={savedColors}
        style={style}
      />
    </>
  );
}
