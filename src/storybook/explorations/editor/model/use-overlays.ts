// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";

import { type CaptionProfanity } from "../../../../bindings/CaptionProfanity";
import { type KeyboardEffectAnimation } from "../../../../bindings/KeyboardEffectAnimation";
import { type KeyboardEffectAppearance } from "../../../../bindings/KeyboardEffectAppearance";
import { type OverlaySize } from "../../../../bindings/OverlaySize";
import { type Placement } from "../../../../bindings/Placement";
import { type TextLines } from "../../../../bindings/TextLines";
import { type TextSize } from "../../../../bindings/TextSize";
import {
  STORY_TEXT_TEMPLATES,
  storyTextStyle,
} from "../../../../features/editor/tool-panels/text-style/text-template-fixtures";

// How the overlays drawn over the whole recording look: the captions and
// the shortcuts, with the app's own options for each. Choosing a caption
// template takes up its layout, as the app does for options not set by hand.

export type CaptionSettings = {
  allCaps: boolean;
  hideFillers: boolean;
  hidePunctuation: boolean;
  holdUntilNext: boolean;
  lines: TextLines;
  maxWords: number;
  position: Placement;
  /** `null` shows profanity as it was said. */
  profanity: CaptionProfanity | null;
  size: TextSize;
  template: string;
};

export type KeysSettings = {
  animation: KeyboardEffectAnimation;
  appearance: KeyboardEffectAppearance;
  position: Placement;
  size: OverlaySize;
};

/** The layout a template brings with it. */
const layoutOf = (template: string) => {
  const style = storyTextStyle(template);
  return {
    allCaps: style.allCaps,
    lines: style.lines,
    maxWords: style.maxWords,
    position: style.position,
    size: style.size,
  };
};

export function useOverlays() {
  const [captions, setCaptions] = useState<CaptionSettings>(() => ({
    hideFillers: false,
    hidePunctuation: false,
    holdUntilNext: false,
    profanity: null,
    template: STORY_TEXT_TEMPLATES.defaultTemplate,
    ...layoutOf(STORY_TEXT_TEMPLATES.defaultTemplate),
  }));
  const [keys, setKeys] = useState<KeysSettings>({
    animation: "pop",
    appearance: "dark",
    position: "bottom",
    size: "standard",
  });
  return {
    captions,
    keys,
    /** Changes the captions; a new template brings its layout along. */
    setCaptions: (change: Partial<CaptionSettings>) => {
      setCaptions((current) => ({
        ...current,
        ...(change.template ? layoutOf(change.template) : null),
        ...change,
      }));
    },
    setKeys: (change: Partial<KeysSettings>) => {
      setKeys((current) => ({ ...current, ...change }));
    },
  };
}
