// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { TextColorRows } from "./text-color-rows";
import { TextEffectsRow } from "./text-effects-row";
import { TextFontRow } from "./text-font-row";
import { TextShadowRow } from "./text-shadow-row";

import type { TextStyle } from "../../../../bindings/TextStyle";

/**
 * A caption template's look, which it sets and saves as its default: the
 * faces, with the accent's beside the main one where the template sets a
 * key word apart, which way the words' shadow falls where they cast one,
 * the effects it draws, and the colours. How the words are arranged in
 * lines is the template's own. How large, where and how many words at a
 * time are the Layout tab's.
 */
export function TextStyleRows({
  isDisabled,
  onChange,
  savedColors,
  style,
}: {
  isDisabled: boolean;
  onChange: (style: Partial<TextStyle>) => void;
  savedColors: string[];
  style: TextStyle;
}) {
  return (
    <>
      <TextFontRow
        font={style.font}
        isDisabled={isDisabled}
        onChange={(font) => {
          onChange({ font });
        }}
      />

      {style.accent === "word" ? (
        <TextFontRow
          font={style.accentFont}
          isAccent
          isDisabled={isDisabled}
          onChange={(accentFont) => {
            onChange({ accentFont });
          }}
        />
      ) : null}

      {style.shadow ? (
        <TextShadowRow
          isDisabled={isDisabled}
          onChange={(shadow) => {
            onChange({ shadow });
          }}
          shadow={style.shadow}
        />
      ) : null}

      <TextEffectsRow
        isDisabled={isDisabled}
        onChange={onChange}
        style={style}
      />

      <TextColorRows
        isDisabled={isDisabled}
        onChange={onChange}
        savedColors={savedColors}
        style={style}
      />
    </>
  );
}
