// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useMemo, useState } from "react";

import { ControlRow } from "../../../../components/shared/control-row/control-row";
import { t } from "../../../../i18n/i18n";
import { PopupSelect } from "../../../popup-panel/popup-select";
import { PopupPanelItem } from "../../../popup-panel/store";
import { installedFontFamilies } from "../../recording/captions/captions-api";

import type { CaptionFont } from "../../../../bindings/CaptionFont";
import type { FontFamily } from "../../../../bindings/FontFamily";

/** The list's key for the app's own Inter, which the style keeps as no
 * family; no installed family is named with a control character. */
const BUILT_IN = "\u0000built-in";
const BUILT_IN_WEIGHTS = [100, 200, 300, 400, 500, 600, 700, 800, 900];
/** Room in the font list for a long family name beside its check. */
const FONT_LIST_WIDTH = 260;

/** The weight nearest `wanted` that `weights` holds. */
const nearest = (weights: number[], wanted: number) =>
  weights.reduce(
    (best, weight) =>
      Math.abs(weight - wanted) < Math.abs(best - wanted) ? weight : best,
    weights[0] ?? wanted,
  );

/** The name a weight goes by, read from the hundred nearest it. */
const weightName = (weight: number) => {
  const names = [
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

/** The families installed on the system, once read. */
function useFontFamilies() {
  const [families, setFamilies] = useState<FontFamily[]>([]);
  useEffect(() => {
    let current = true;
    void installedFontFamilies()
      .then((found) => {
        if (current) setFamilies(found);
      })
      // Without the system's list, the app's own Inter is still offered.
      .catch(() => undefined);
    return () => {
      current = false;
    };
  }, []);
  return families;
}

/**
 * The face captions are set in, chosen the way a system font panel chooses
 * one: a family, from every one installed, found by typing and each named
 * in its own type, and beside it the weight, from those the family has.
 */
export function CaptionFontRow({
  font,
  isDisabled,
  onChange,
}: {
  font: CaptionFont;
  isDisabled: boolean;
  onChange: (font: CaptionFont) => void;
}) {
  const families = useFontFamilies();
  const builtIn = t("editor-panels-captions-font-built-in");
  const choices = useMemo(
    () => [
      {
        family: "Inter",
        id: BUILT_IN,
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
  // Weights that read the same are told apart by their number.
  const weightItems: PopupPanelItem[] = chosen.weights.map((weight) => {
    const label = weightName(weight);
    const shared = chosen.weights.some(
      (other) => other !== weight && weightName(other) === label,
    );
    return {
      ...(shared ? { detail: weight.toString() } : {}),
      font: { family: chosen.family, weight },
      id: weight.toString(),
      label,
    };
  });

  // Both pickers keep one width whatever they show, so choosing another
  // font or weight never shifts the row; long names truncate instead.
  return (
    <ControlRow
      controlClassName="gap-control"
      title={t("editor-panels-captions-font")}
    >
      {(controlProps) => (
        <div
          {...controlProps}
          className="flex items-center gap-control"
          role="group"
        >
          <PopupSelect
            className="w-36"
            id="caption-font"
            isDisabled={isDisabled}
            items={fontItems}
            label={t("editor-panels-captions-font")}
            minimumListWidth={FONT_LIST_WIDTH}
            onSelectionChange={(item) => {
              const family = choices.find((choice) => choice.id === item.id);
              if (!family) return;
              onChange({
                ...(family.id === BUILT_IN ? {} : { family: family.name }),
                weight: nearest(family.weights, font.weight),
              });
            }}
            placeholder={t("editor-panels-captions-font")}
            searchLabel={t("editor-panels-captions-font-search")}
            selectedId={chosen.id}
          />
          <PopupSelect
            className="w-28"
            id="caption-weight"
            isDisabled={isDisabled || weightItems.length < 2}
            items={weightItems}
            label={t("editor-panels-captions-weight")}
            onSelectionChange={(item) => {
              onChange({ ...font, weight: Number(item.id) });
            }}
            placeholder={t("editor-panels-captions-weight")}
            selectedId={nearest(chosen.weights, font.weight).toString()}
          />
        </div>
      )}
    </ControlRow>
  );
}
