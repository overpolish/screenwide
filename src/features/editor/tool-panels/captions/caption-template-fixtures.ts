// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { DEFAULT_CAPTION_SETTINGS } from "../../recording/captions/recording-captions";

import type { CaptionStyle } from "../../../../bindings/CaptionStyle";
import type { CaptionTemplates } from "../../recording/captions/use-caption-templates";

/** The templates as `CaptionTemplate::style` ships them, for stories, which
 * have no app to ask. */
const plain: CaptionStyle = {
  accent: "none",
  accentColor: "#ffd60a",
  accentFont: { family: "Instrument Serif", italic: true, weight: 400 },
  accentSize: "medium",
  accentSizePercent: 120,
  allCaps: false,
  arrangement: "lines",
  background: "none",
  backgroundColor: "#0000009e",
  entrance: "none",
  font: { italic: false, weight: 600 },
  highlightColor: "#ffd60a",
  highlightWords: false,
  lines: "two",
  maxWords: 8,
  outline: true,
  outlineColor: "#000000",
  position: "bottom",
  revealWords: false,
  size: "medium",
  sizePercent: 100,
  template: "clean",
  textColor: "#ffffff",
};

const bold = {
  allCaps: true,
  font: { italic: false, weight: 800 },
  lines: "one",
} as const;

/** `family` at `weight`, upright. */
const upright = (family: string, weight: number) => ({
  family,
  italic: false,
  weight,
});

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
    highlightWords: true,
    maxWords: 4,
    sizePercent: 120,
    template: "highlight",
  },
  {
    ...plain,
    allCaps: true,
    font: upright("Montserrat", 800),
    highlightColor: "#4ade80",
    highlightWords: true,
    maxWords: 6,
    sizePercent: 110,
    template: "karaoke",
  },
  {
    ...plain,
    allCaps: true,
    entrance: "pop",
    font: upright("Anton", 400),
    highlightWords: true,
    lines: "one",
    maxWords: 2,
    position: "center",
    sizePercent: 170,
    template: "punch",
  },
  {
    ...plain,
    accent: "word",
    accentColor: "#ffffff",
    accentSizePercent: 130,
    font: upright("DM Sans", 600),
    maxWords: 6,
    sizePercent: 110,
    template: "editorial",
  },
  {
    ...plain,
    accent: "word",
    accentFont: upright("Montserrat", 900),
    accentSizePercent: 170,
    allCaps: true,
    entrance: "pop",
    font: upright("Montserrat", 900),
    maxWords: 5,
    sizePercent: 110,
    template: "supersize",
  },
  {
    ...plain,
    accent: "word",
    accentFont: upright("Anton", 400),
    accentSizePercent: 200,
    allCaps: true,
    arrangement: "stack",
    font: upright("Montserrat", 800),
    maxWords: 6,
    position: "center",
    revealWords: true,
    sizePercent: 110,
    template: "stack",
  },
  {
    ...plain,
    accent: "word",
    accentColor: "#ffffff",
    accentSizePercent: 135,
    arrangement: "staircase",
    font: upright("DM Sans", 700),
    maxWords: 6,
    position: "center",
    revealWords: true,
    sizePercent: 115,
    template: "staircase",
  },
];

export const STORY_CAPTION_TEMPLATES: CaptionTemplates = {
  defaultLayout: {},
  defaultSettings: DEFAULT_CAPTION_SETTINGS,
  defaultTemplate: "box",
  shipped: STORY_CAPTION_STYLES,
  styles: STORY_CAPTION_STYLES,
};
