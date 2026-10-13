// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { DEFAULT_CAPTION_SETTINGS } from "../../recording/captions/recording-captions";

import type { CaptionStyle } from "../../../../bindings/CaptionStyle";
import type { CaptionTemplates } from "../../recording/captions/use-caption-templates";

/** The templates as `CaptionTemplate::style` ships them, for stories, which
 * have no app to ask. */
const plain: CaptionStyle = {
  allCaps: false,
  background: "none",
  backgroundColor: "#0000009e",
  entrance: "none",
  font: { weight: 600 },
  highlightColor: "#ffd60a",
  highlightWords: false,
  lines: "two",
  maxWords: 8,
  outline: true,
  outlineColor: "#000000",
  position: "bottom",
  sizePercent: 100,
  template: "clean",
  textColor: "#ffffff",
};

const bold = {
  allCaps: true,
  font: { weight: 800 },
  lines: "one",
} as const;

export const STORY_CAPTION_STYLES: CaptionStyle[] = [
  { ...plain, background: "box", outline: false, template: "box" },
  plain,
  {
    ...plain,
    ...bold,
    entrance: "pop",
    maxWords: 3,
    sizePercent: 140,
    template: "pop",
  },
  {
    ...plain,
    ...bold,
    background: "word",
    maxWords: 4,
    sizePercent: 120,
    template: "highlight",
  },
  {
    ...plain,
    entrance: "words",
    maxWords: 6,
    sizePercent: 110,
    template: "reveal",
  },
];

export const STORY_CAPTION_TEMPLATES: CaptionTemplates = {
  defaultLayout: {},
  defaultSettings: DEFAULT_CAPTION_SETTINGS,
  defaultTemplate: "box",
  shipped: STORY_CAPTION_STYLES,
  styles: STORY_CAPTION_STYLES,
};
