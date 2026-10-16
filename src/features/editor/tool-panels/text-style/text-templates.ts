// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../../i18n/i18n";

import type { TextStyle } from "../../../../bindings/TextStyle";
import type { TextTemplateId } from "../../../../bindings/TextTemplateId";

/** A caption template's name: one the app ships, in the app's own
 * translations, keyed by its id; any other's as its manifest gives it,
 * `named`, or its id where there is none. */
export const textTemplateLabel = (template: TextTemplateId, named?: string) => {
  const bundled: Partial<Record<TextTemplateId, string>> = {
    box: t("editor-panels-text-template-box"),
    clean: t("editor-panels-text-template-clean"),
    editorial: t("editor-panels-text-template-editorial"),
    flip: t("editor-panels-text-template-flip"),
    glow: t("editor-panels-text-template-glow"),
    highlight: t("editor-panels-text-template-highlight"),
    karaoke: t("editor-panels-text-template-karaoke"),
    offset: t("editor-panels-text-template-offset"),
    pop: t("editor-panels-text-template-pop"),
    punch: t("editor-panels-text-template-punch"),
    rise: t("editor-panels-text-template-rise"),
    shadow: t("editor-panels-text-template-shadow"),
    stack: t("editor-panels-text-template-stack"),
    stagger: t("editor-panels-text-template-stagger"),
    staircase: t("editor-panels-text-template-staircase"),
    supersize: t("editor-panels-text-template-supersize"),
    underline: t("editor-panels-text-template-underline"),
  };
  return bundled[template] ?? named ?? template;
};

/** Whether `style` can outline its type: one set on a box reads without,
 * and an outline over the box only thickens the type. */
export const offersOutline = (style: TextStyle) => style.background !== "box";

/** A colour a caption style is drawn in, by what it colours. */
export type TextColorRole =
  "accent" | "background" | "highlight" | "outline" | "shadow" | "text";

/** The style's field each colour but the shadow's is kept in. */
const CAPTION_COLOR_KEYS = {
  accent: "accentColor",
  background: "backgroundColor",
  highlight: "highlightColor",
  outline: "outlineColor",
  text: "textColor",
} as const satisfies Record<Exclude<TextColorRole, "shadow">, keyof TextStyle>;

/** The colour `style` draws `role` in. A shadow's is given without how
 * see-through it is, which is the template's to set and survives a change
 * of colour: so its well shows the colour picked, and the grid shows the
 * swatch it came from as chosen. Type set to read on its box shows the black
 * or white it is drawn in, which picking a colour then replaces. */
export const styleColor = (style: TextStyle, role: TextColorRole) => {
  if (role === "shadow") return (style.shadow?.color ?? "#000000").slice(0, 7);
  const color = style[CAPTION_COLOR_KEYS[role]];
  return role === "text" && color === AUTO_INK
    ? autoInk(style.backgroundColor)
    : color;
};

/** The text colour that is black or white, whichever reads on the box: the
 * twin of `AUTO_INK` in Rust's text crate. */
const AUTO_INK = "auto";

/** Type that reads on `color` by the counter's rule: black on a light
 * colour, white on a dark one. The twin of `auto_ink` in Rust's text crate. */
const autoInk = (color: string) => {
  const channel = (at: number) => parseInt(color.slice(at, at + 2), 16) / 255;
  const luminance =
    0.2126 * channel(1) + 0.7152 * channel(3) + 0.0722 * channel(5);
  return luminance > 0.6 ? "#000000" : "#ffffff";
};

/** The change to `style` that draws `role` in `color`. A shadow keeps how
 * see-through it is, which the palette's colours do not say. */
export const withStyleColor = (
  style: TextStyle,
  role: TextColorRole,
  color: string,
): Partial<TextStyle> => {
  if (role !== "shadow") return { [CAPTION_COLOR_KEYS[role]]: color };
  if (!style.shadow) return {};
  const alpha =
    style.shadow.color.length === 9 ? style.shadow.color.slice(7) : "";
  const shadowColor = color.length === 7 ? color + alpha : color;
  return { shadow: { ...style.shadow, color: shadowColor } };
};

/** The colours `style` draws in, so the ones it offers to change: its type
 * always, the word being said where it is marked, the accent where it sets
 * words apart, its outline, shadow and box where it has them. */
export const offeredColors = (style: TextStyle): TextColorRole[] => [
  "text",
  ...(style.highlightWords ? (["highlight"] as const) : []),
  ...(style.accent === "none" ? [] : (["accent"] as const)),
  ...(style.outline && offersOutline(style) ? (["outline"] as const) : []),
  ...(style.shadow ? (["shadow"] as const) : []),
  ...(style.background === "box" ? (["background"] as const) : []),
];

/** A colour's name, as its pill shows it. */
export const styleColorLabel = (role: TextColorRole) =>
  ({
    accent: t("editor-panels-text-color-accent"),
    background: t("editor-panels-text-color-background"),
    highlight: t("editor-panels-text-color-highlight"),
    outline: t("editor-panels-text-color-outline"),
    shadow: t("editor-panels-text-color-shadow"),
    text: t("editor-panels-text-color-text"),
  })[role];
