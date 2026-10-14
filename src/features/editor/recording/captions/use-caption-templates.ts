// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useMemo, useState } from "react";

import { layoutOf, withLayout } from "./caption-layout";
import {
  captionTemplates,
  followCaptionTemplates,
  forgetCaptionTemplates,
} from "./captions-api";
import { useCaptionTemplateFonts } from "./use-caption-template-fonts";

import type { CaptionLayout } from "../../../../bindings/CaptionLayout";
import type { CaptionSettings } from "../../../../bindings/CaptionSettings";
import type { CaptionStyle } from "../../../../bindings/CaptionStyle";
import type { CaptionTemplate } from "../../../../bindings/CaptionTemplate";
import type { CaptionTemplateId } from "../../../../bindings/CaptionTemplateId";
import type { CaptionTemplateSource } from "../../../../bindings/CaptionTemplateSource";

/** The caption templates as the panel offers them, and what new captions
 * start as: the template, its look, the layout options set by hand, and the
 * rules for what they say. */
export type CaptionTemplates = {
  /** The layout options new captions keep over any template. */
  defaultLayout: CaptionLayout;
  /** What new captions start out saying. */
  defaultSettings: CaptionSettings;
  defaultTemplate: CaptionTemplateId;
  /** Each template's name, as its manifest gives it. */
  names: Partial<Record<CaptionTemplateId, string>>;
  /** Each template as it ships, in the order the panel offers them. */
  shipped: CaptionStyle[];
  /** Whether each template is the app's own or one of yours. */
  sources: Partial<Record<CaptionTemplateId, CaptionTemplateSource>>;
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
  original?: CaptionTemplateId;
  settings?: CaptionSettings;
  style?: CaptionStyle;
};

/**
 * The caption templates with the defaults the general settings keep laid
 * over the ones they ship with, asked for again whenever yours change. None
 * until they arrive.
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
    template: CaptionTemplateId;
  },
): CaptionTemplates | null {
  const [catalogue, setCatalogue] = useState<CaptionTemplate[] | null>(null);
  useCaptionTemplateFonts();
  useEffect(() => {
    let current = true;
    const ask = () => {
      void captionTemplates().then((templates) => {
        if (current) setCatalogue(templates);
      });
    };
    ask();
    const stop = followCaptionTemplates(() => {
      forgetCaptionTemplates();
      ask();
    });
    return () => {
      current = false;
      stop();
    };
  }, []);
  return useMemo(() => {
    if (!catalogue) return null;
    const shipped = catalogue.map((template) => template.style);
    return {
      defaultLayout,
      defaultSettings,
      defaultTemplate,
      names: Object.fromEntries(
        catalogue.map((template) => [template.id, template.info.name]),
      ),
      shipped,
      sources: Object.fromEntries(
        catalogue.map((template) => [template.id, template.source]),
      ),
      styles: shipped.map((style) => {
        const saved = savedStyles.find(
          (each) => each.template === style.template,
        );
        return saved ? withLayout(layoutOf(style), saved) : style;
      }),
    };
  }, [catalogue, defaultLayout, defaultSettings, defaultTemplate, savedStyles]);
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
  captionTemplate?: CaptionTemplateId;
} => {
  const { layout, original, settings, style } = patch;
  const others = (template: CaptionTemplateId) =>
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
