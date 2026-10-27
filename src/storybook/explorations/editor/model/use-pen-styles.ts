// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";

import { type Pen } from "../../ideas/looks";

import { type AnnotationStyle, DEFAULT_STYLE } from "./annotation-style";

// The style each pen draws new annotations in, set from the pen's island,
// and the colours kept beside the palette, shared by every colour grid.

export function usePenStyles() {
  const [penStyles, setPenStyles] = useState<Record<Pen, AnnotationStyle>>({
    arrow: DEFAULT_STYLE,
    counter: DEFAULT_STYLE,
    focus: DEFAULT_STYLE,
  });
  const [savedColors, setSavedColors] = useState<string[]>([]);
  return {
    penStyles,
    removeColor: (color: string) => {
      setSavedColors((current) => current.filter((kept) => kept !== color));
    },
    saveColor: (color: string) => {
      setSavedColors((current) =>
        current.includes(color) ? current : [...current, color],
      );
    },
    savedColors,
    setPenStyle: (pen: Pen, change: Partial<AnnotationStyle>) => {
      setPenStyles((current) => ({
        ...current,
        [pen]: { ...current[pen], ...change },
      }));
    },
  };
}
