// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useMemo, useState } from "react";

import { t } from "../../../../i18n/i18n";
import {
  followTextTemplates,
  forgetFontFamilies,
  installedFontFamilies,
} from "../../recording/captions/captions-api";

import type { FontFamily } from "../../../../bindings/FontFamily";
import type { TextFont } from "../../../../bindings/TextFont";

// The faces text is set in, as every picker offers them: the app's own
// Inter first, then each family installed on the system or brought by a
// template, and for the family in hand its weights, upright and, where it
// has one, italic.

/** The list's key for the app's own Inter, which the style keeps as no
 * family; no installed family is named with a control character. */
const BUILT_IN = "\u0000built-in";
const BUILT_IN_WEIGHTS = [100, 200, 300, 400, 500, 600, 700, 800, 900];
/** What a weight's key marks an italic by, after its weight. */
const ITALIC = "i";
/** No families beyond the system's, one list for every render. */
const NO_FAMILIES: FontFamily[] = [];

/** A family a picker offers: its key, the family to set it in, its name,
 * whether it has an italic, and its weights. */
export type FontChoice = {
  family: string;
  id: string;
  italic: boolean;
  name: string;
  weights: number[];
};

/** One of a family's styles: a weight, upright or italic, named as a font
 * panel names it, with its number where another reads the same. */
export type FontStyleChoice = {
  font: TextFont;
  id: string;
  label: string;
  detail?: string;
};

/** The weight nearest `wanted` that `weights` holds. */
const nearestWeight = (weights: number[], wanted: number) =>
  weights.reduce(
    (best, weight) =>
      Math.abs(weight - wanted) < Math.abs(best - wanted) ? weight : best,
    weights[0] ?? wanted,
  );

/** The name a weight goes by, upright or italic, read from the hundred
 * nearest it. */
const weightName = (weight: number, italic: boolean) => {
  const names = italic
    ? [
        () => t("editor-panels-text-weight-thin-italic"),
        () => t("editor-panels-text-weight-extra-light-italic"),
        () => t("editor-panels-text-weight-light-italic"),
        () => t("editor-panels-text-weight-regular-italic"),
        () => t("editor-panels-text-weight-medium-italic"),
        () => t("editor-panels-text-weight-semibold-italic"),
        () => t("editor-panels-text-weight-bold-italic"),
        () => t("editor-panels-text-weight-extra-bold-italic"),
        () => t("editor-panels-text-weight-black-italic"),
      ]
    : [
        () => t("editor-panels-text-weight-thin"),
        () => t("editor-panels-text-weight-extra-light"),
        () => t("editor-panels-text-weight-light"),
        () => t("editor-panels-text-weight-regular"),
        () => t("editor-panels-text-weight-medium"),
        () => t("editor-panels-text-weight-semibold"),
        () => t("editor-panels-text-weight-bold"),
        () => t("editor-panels-text-weight-extra-bold"),
        () => t("editor-panels-text-weight-black"),
      ];
  const hundred = Math.min(Math.max(Math.round(weight / 100), 1), 9);
  return names[hundred - 1]();
};

/** The families installed on the system or brought by a template, once
 * read, and read again when your templates change. */
function useFontFamilies() {
  const [families, setFamilies] = useState<FontFamily[]>([]);
  useEffect(() => {
    let current = true;
    const ask = () => {
      void installedFontFamilies()
        .then((found) => {
          if (current) setFamilies(found);
        })
        // Without the system's list, the app's own Inter is still offered.
        .catch(() => undefined);
    };
    ask();
    const stop = followTextTemplates(() => {
      forgetFontFamilies();
      ask();
    });
    return () => {
      current = false;
      stop();
    };
  }, []);
  return families;
}

/** Every family a picker offers, the app's own Inter first; with `extra`,
 * families a story offers where there is no system to ask. */
export function useFontChoices(
  extra: FontFamily[] = NO_FAMILIES,
): FontChoice[] {
  const installed = useFontFamilies();
  const builtIn = t("editor-panels-text-font-built-in");
  return useMemo(
    () => [
      {
        family: "Inter",
        id: BUILT_IN,
        italic: false,
        name: builtIn,
        weights: BUILT_IN_WEIGHTS,
      },
      ...[...installed, ...extra].map((family) => ({
        family: family.name,
        id: family.name,
        ...family,
      })),
    ],
    [builtIn, extra, installed],
  );
}

/** The family `font` is set in, of `choices`, or the first where it is not
 * there. */
export const chosenFontFamily = (choices: FontChoice[], font: TextFont) =>
  choices.find((choice) => choice.id === (font.family ?? BUILT_IN)) ??
  choices[0];

/** `font` moved to `family`: its weight the nearest that family has, and
 * italic only where the family has one. */
export const fontInFamily = (font: TextFont, family: FontChoice): TextFont => ({
  ...(family.id === BUILT_IN ? {} : { family: family.name }),
  italic: font.italic && family.italic,
  weight: nearestWeight(family.weights, font.weight),
});

/** `family`'s styles: each weight upright, then italic beside it where the
 * family has one; weights that read the same are told apart by their
 * number. */
export const fontStyles = (family: FontChoice): FontStyleChoice[] =>
  family.weights
    .flatMap((weight) =>
      family.italic
        ? [
            { italic: false, weight },
            { italic: true, weight },
          ]
        : [{ italic: false, weight }],
    )
    .map(({ italic, weight }) => {
      const label = weightName(weight, italic);
      const shared = family.weights.some(
        (other) => other !== weight && weightName(other, italic) === label,
      );
      return {
        ...(shared ? { detail: weight.toString() } : {}),
        font: {
          ...(family.id === BUILT_IN ? {} : { family: family.family }),
          italic,
          weight,
        },
        id: `${weight.toString()}${italic ? ITALIC : ""}`,
        label,
      };
    });

/** The key of the style `font` is set in, within `family`. */
export const fontStyleId = (family: FontChoice, font: TextFont) =>
  `${nearestWeight(family.weights, font.weight).toString()}${
    font.italic && family.italic ? ITALIC : ""
  }`;
