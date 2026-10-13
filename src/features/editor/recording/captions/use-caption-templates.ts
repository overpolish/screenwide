// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useMemo, useState } from "react";

import { layoutOf, withLayout } from "./caption-layout";
import { captionTemplateStyles } from "./captions-api";

import type { CaptionLayout } from "../../../../bindings/CaptionLayout";
import type { CaptionSettings } from "../../../../bindings/CaptionSettings";
import type { CaptionStyle } from "../../../../bindings/CaptionStyle";
import type { CaptionTemplate } from "../../../../bindings/CaptionTemplate";

/** The caption templates as the panel offers them, and what new captions
 * start as: the template, its look, the layout options set by hand, and the
 * rules for what they say. */
export type CaptionTemplates = {
  /** The layout options new captions keep over any template. */
  defaultLayout: CaptionLayout;
  /** What new captions start out saying. */
  defaultSettings: CaptionSettings;
  defaultTemplate: CaptionTemplate;
  /** Each template as it ships, in the order the panel offers them. */
  shipped: CaptionStyle[];
  /** Each template's look as it was last set as default, or as it ships,
   * over the layout options it ships with: only the Layout tab changes
   * those, so a look saved with others never brings them back. */
  styles: CaptionStyle[];
};

/** A change to what new captions start as, kept in the general settings:
 * a template, looking as it does now, set as the one they start with; the
 * layout options they keep; the rules they start with; or a template put
 * back to how it ships. */
export type CaptionDefaultsPatch = {
  layout?: CaptionLayout;
  original?: CaptionTemplate;
  settings?: CaptionSettings;
  style?: CaptionStyle;
};

/**
 * The caption templates with the defaults the general settings keep laid
 * over the ones they ship with. None until the shipped ones arrive.
 */
export function useCaptionTemplates(
  savedStyles: CaptionStyle[],
  {
    layout: defaultLayout,
    settings: defaultSettings,
    template: defaultTemplate,
  }: {
    layout: CaptionLayout;
    settings: CaptionSettings;
    template: CaptionTemplate;
  },
): CaptionTemplates | null {
  const [shipped, setShipped] = useState<CaptionStyle[] | null>(null);
  useEffect(() => {
    let current = true;
    void captionTemplateStyles().then((styles) => {
      if (current) setShipped(styles);
    });
    return () => {
      current = false;
    };
  }, []);
  return useMemo(
    () =>
      shipped && {
        defaultLayout,
        defaultSettings,
        defaultTemplate,
        shipped,
        styles: shipped.map((style) => {
          const saved = savedStyles.find(
            (each) => each.template === style.template,
          );
          return saved ? withLayout(layoutOf(style), saved) : style;
        }),
      },
    [defaultLayout, defaultSettings, defaultTemplate, savedStyles, shipped],
  );
}

/** What new captions start as: the default template, as set, with the
 * layout options kept for them, saying what the default rules have it say.
 * The twin of `default_captions` in the app. */
export const defaultCaptions = (templates: CaptionTemplates) => ({
  layout: templates.defaultLayout,
  settings: templates.defaultSettings,
  style: withLayout(
    templates.defaultLayout,
    templates.styles.find(
      (style) => style.template === templates.defaultTemplate,
    ) ?? templates.styles[0],
  ),
});

/** What `patch` changes in the general settings, from the caption defaults
 * they keep now. */
export const captionDefaultsSettings = (
  saved: CaptionStyle[],
  patch: CaptionDefaultsPatch,
): {
  captionLayout?: CaptionLayout;
  captionSettings?: CaptionSettings;
  captionStyles?: CaptionStyle[];
  captionTemplate?: CaptionTemplate;
} => {
  const { layout, original, settings, style } = patch;
  const others = (template: CaptionTemplate) =>
    saved.filter((kept) => kept.template !== template);
  return {
    ...(layout ? { captionLayout: layout } : {}),
    ...(settings ? { captionSettings: settings } : {}),
    ...(style
      ? {
          captionStyles: [...others(style.template), style],
          captionTemplate: style.template,
        }
      : original
        ? { captionStyles: others(original) }
        : {}),
  };
};
