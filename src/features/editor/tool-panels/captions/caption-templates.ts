// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../../i18n/i18n";

import type { CaptionStyle } from "../../../../bindings/CaptionStyle";
import type { CaptionTemplateId } from "../../../../bindings/CaptionTemplateId";

/** A caption template's name: one the app ships, in the app's own
 * translations, keyed by its id; any other's as its manifest gives it,
 * `named`, or its id where there is none. */
export const captionTemplateLabel = (
  template: CaptionTemplateId,
  named?: string,
) => {
  const bundled: Partial<Record<CaptionTemplateId, string>> = {
    box: t("editor-panels-captions-template-box"),
    clean: t("editor-panels-captions-template-clean"),
    editorial: t("editor-panels-captions-template-editorial"),
    glow: t("editor-panels-captions-template-glow"),
    highlight: t("editor-panels-captions-template-highlight"),
    karaoke: t("editor-panels-captions-template-karaoke"),
    offset: t("editor-panels-captions-template-offset"),
    pop: t("editor-panels-captions-template-pop"),
    punch: t("editor-panels-captions-template-punch"),
    rise: t("editor-panels-captions-template-rise"),
    shadow: t("editor-panels-captions-template-shadow"),
    stack: t("editor-panels-captions-template-stack"),
    stagger: t("editor-panels-captions-template-stagger"),
    staircase: t("editor-panels-captions-template-staircase"),
    supersize: t("editor-panels-captions-template-supersize"),
    underline: t("editor-panels-captions-template-underline"),
  };
  return bundled[template] ?? named ?? template;
};

/** Whether `style` can outline its type: one set on a box reads without,
 * and an outline over the box only thickens the type. */
export const offersOutline = (style: CaptionStyle) =>
  style.background !== "box";

/** A colour a caption style is drawn in, by what it colours. */
export type CaptionColorRole =
  "accent" | "background" | "highlight" | "outline" | "shadow" | "text";

/** The style's field each colour but the shadow's is kept in. */
const CAPTION_COLOR_KEYS = {
  accent: "accentColor",
  background: "backgroundColor",
  highlight: "highlightColor",
  outline: "outlineColor",
  text: "textColor",
} as const satisfies Record<
  Exclude<CaptionColorRole, "shadow">,
  keyof CaptionStyle
>;

/** The colour `style` draws `role` in. A shadow's is given without how
 * see-through it is, which is the template's to set and survives a change
 * of colour: so its well shows the colour picked, and the grid shows the
 * swatch it came from as chosen. */
export const captionColor = (style: CaptionStyle, role: CaptionColorRole) =>
  role === "shadow"
    ? (style.shadow?.color ?? "#000000").slice(0, 7)
    : style[CAPTION_COLOR_KEYS[role]];

/** The change to `style` that draws `role` in `color`. A shadow keeps how
 * see-through it is, which the palette's colours do not say. */
export const withCaptionColor = (
  style: CaptionStyle,
  role: CaptionColorRole,
  color: string,
): Partial<CaptionStyle> => {
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
export const offeredColors = (style: CaptionStyle): CaptionColorRole[] => [
  "text",
  ...(style.highlightWords ? (["highlight"] as const) : []),
  ...(style.accent === "none" ? [] : (["accent"] as const)),
  ...(style.outline && offersOutline(style) ? (["outline"] as const) : []),
  ...(style.shadow ? (["shadow"] as const) : []),
  ...(style.background === "box" ? (["background"] as const) : []),
];

/** A colour's name, as its pill shows it. */
export const captionColorLabel = (role: CaptionColorRole) =>
  ({
    accent: t("editor-panels-captions-color-accent"),
    background: t("editor-panels-captions-color-background"),
    highlight: t("editor-panels-captions-color-highlight"),
    outline: t("editor-panels-captions-color-outline"),
    shadow: t("editor-panels-captions-color-shadow"),
    text: t("editor-panels-captions-color-text"),
  })[role];
