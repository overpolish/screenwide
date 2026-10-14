// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useMemo, useState } from "react";

import { ControlRow } from "../../../../components/shared/control-row/control-row";
import { t } from "../../../../i18n/i18n";
import { PopupSelect } from "../../../popup-panel/popup-select";
import { PopupPanelItem } from "../../../popup-panel/store";
import {
  followCaptionTemplates,
  forgetFontFamilies,
  installedFontFamilies,
} from "../../recording/captions/captions-api";

import {
  CAPTION_FAMILY_WIDTH,
  CAPTION_WEIGHT_WIDTH,
} from "./caption-field-widths";

import type { CaptionFont } from "../../../../bindings/CaptionFont";
import type { FontFamily } from "../../../../bindings/FontFamily";

/** The list's key for the app's own Inter, which the style keeps as no
 * family; no installed family is named with a control character. */
const BUILT_IN = "\u0000built-in";
const BUILT_IN_WEIGHTS = [100, 200, 300, 400, 500, 600, 700, 800, 900];
/** Room in the font list for a long family name beside its check. */
const FONT_LIST_WIDTH = 260;
/** What the weight list keys an italic by, after its weight. */
const ITALIC = "i";

/** The weight nearest `wanted` that `weights` holds. */
const nearest = (weights: number[], wanted: number) =>
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
        () => t("editor-panels-captions-weight-thin-italic"),
        () => t("editor-panels-captions-weight-extra-light-italic"),
        () => t("editor-panels-captions-weight-light-italic"),
        () => t("editor-panels-captions-weight-regular-italic"),
        () => t("editor-panels-captions-weight-medium-italic"),
        () => t("editor-panels-captions-weight-semibold-italic"),
        () => t("editor-panels-captions-weight-bold-italic"),
        () => t("editor-panels-captions-weight-extra-bold-italic"),
        () => t("editor-panels-captions-weight-black-italic"),
      ]
    : [
        () => t("editor-panels-captions-weight-thin"),
        () => t("editor-panels-captions-weight-extra-light"),
        () => t("editor-panels-captions-weight-light"),
        () => t("editor-panels-captions-weight-regular"),
        () => t("editor-panels-captions-weight-medium"),
        () => t("editor-panels-captions-weight-semibold"),
        () => t("editor-panels-captions-weight-bold"),
        () => t("editor-panels-captions-weight-extra-bold"),
        () => t("editor-panels-captions-weight-black"),
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
    const stop = followCaptionTemplates(() => {
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

/**
 * A face captions are set in, chosen the way a system font panel chooses
 * one: a family, from every one installed or carried by the app, found by
 * typing and each named in its own type, and beside it the style, from the
 * weights the family has, upright and, where it has one, italic. The main
 * face and the accent's each have one.
 */
export function CaptionFontRow({
  font,
  isAccent = false,
  isDisabled,
  onChange,
}: {
  font: CaptionFont;
  isDisabled: boolean;
  onChange: (font: CaptionFont) => void;
  isAccent?: boolean;
}) {
  const families = useFontFamilies();
  const builtIn = t("editor-panels-captions-font-built-in");
  const choices = useMemo(
    () => [
      {
        family: "Inter",
        id: BUILT_IN,
        italic: false,
        name: builtIn,
        weights: BUILT_IN_WEIGHTS,
      },
      ...families.map((family) => ({
        family: family.name,
        id: family.name,
        ...family,
      })),
    ],
    [builtIn, families],
  );
  const fontItems = useMemo<PopupPanelItem[]>(
    () =>
      choices.map((choice) => ({
        font: { family: choice.family },
        id: choice.id,
        label: choice.name,
      })),
    [choices],
  );
  const chosen =
    choices.find((choice) => choice.id === (font.family ?? BUILT_IN)) ??
    choices[0];
  // Each weight upright, then italic beside it where the family has one;
  // weights that read the same are told apart by their number.
  const styles = chosen.weights.flatMap((weight) =>
    chosen.italic
      ? [
          { italic: false, weight },
          { italic: true, weight },
        ]
      : [{ italic: false, weight }],
  );
  const weightItems: PopupPanelItem[] = styles.map(({ italic, weight }) => {
    const label = weightName(weight, italic);
    const shared = chosen.weights.some(
      (other) => other !== weight && weightName(other, italic) === label,
    );
    return {
      ...(shared ? { detail: weight.toString() } : {}),
      font: { family: chosen.family, italic, weight },
      id: `${weight.toString()}${italic ? ITALIC : ""}`,
      label,
    };
  });
  const title = isAccent
    ? t("editor-panels-captions-accent")
    : t("editor-panels-captions-font");
  const weightLabel = isAccent
    ? t("editor-panels-captions-accent-weight")
    : t("editor-panels-captions-weight");
  const chosenWeight = nearest(chosen.weights, font.weight);

  // Both pickers keep one width whatever they show, so choosing another
  // font or weight never shifts the row; long names truncate instead.
  return (
    <ControlRow controlClassName="gap-control" title={title}>
      {(controlProps) => (
        <div
          {...controlProps}
          className="flex items-center gap-control"
          role="group"
        >
          <PopupSelect
            className={CAPTION_FAMILY_WIDTH}
            id={isAccent ? "caption-accent-font" : "caption-font"}
            isDisabled={isDisabled}
            items={fontItems}
            label={title}
            minimumListWidth={FONT_LIST_WIDTH}
            onSelectionChange={(item) => {
              const family = choices.find((choice) => choice.id === item.id);
              if (!family) return;
              onChange({
                ...(family.id === BUILT_IN ? {} : { family: family.name }),
                italic: font.italic && family.italic,
                weight: nearest(family.weights, font.weight),
              });
            }}
            placeholder={title}
            searchLabel={t("editor-panels-captions-font-search")}
            selectedId={chosen.id}
          />
          <PopupSelect
            className={CAPTION_WEIGHT_WIDTH}
            id={isAccent ? "caption-accent-weight" : "caption-weight"}
            isDisabled={isDisabled || weightItems.length < 2}
            items={weightItems}
            label={weightLabel}
            onSelectionChange={(item) => {
              onChange({
                ...font,
                italic: item.id.endsWith(ITALIC),
                weight: Number.parseInt(item.id, 10),
              });
            }}
            placeholder={weightLabel}
            selectedId={`${chosenWeight.toString()}${
              font.italic && chosen.italic ? ITALIC : ""
            }`}
          />
        </div>
      )}
    </ControlRow>
  );
}
