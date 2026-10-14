// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { CaptionColorRows } from "./caption-color-rows";
import { CaptionEffectsRow } from "./caption-effects-row";
import { CaptionFontRow } from "./caption-font-row";

import type { CaptionStyle } from "../../../../bindings/CaptionStyle";

/**
 * A caption template's look, which it sets and saves as its default: the
 * faces, with the accent's beside the main one where the template sets a
 * key word apart, the effects it draws, and the colours. How the words are
 * arranged in lines is the template's own. How large, where and how many
 * words at a time are the Layout tab's.
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

      {style.accent === "word" ? (
        <CaptionFontRow
          font={style.accentFont}
          isAccent
          isDisabled={isDisabled}
          onChange={(accentFont) => {
            onChange({ accentFont });
          }}
        />
      ) : null}

      <CaptionEffectsRow
        isDisabled={isDisabled}
        onChange={onChange}
        style={style}
      />

      <CaptionColorRows
        isDisabled={isDisabled}
        onChange={onChange}
        savedColors={savedColors}
        style={style}
      />
    </>
  );
}
