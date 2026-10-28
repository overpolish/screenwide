// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useMemo } from "react";

import { ControlRow } from "../../../../composites/control-row/control-row";
import { t } from "../../../../i18n/i18n";
import { PopupSelect } from "../../../popup-panel/popup-select";
import { PopupPanelItem } from "../../../popup-panel/store";

import { TEXT_FAMILY_WIDTH, TEXT_WEIGHT_WIDTH } from "./text-field-widths";
import {
  chosenFontFamily,
  fontInFamily,
  fontStyleId,
  fontStyles,
  useFontChoices,
} from "./text-fonts";

import type { TextFont } from "../../../../bindings/TextFont";

/** Room in the font list for a long family name beside its check. */
const FONT_LIST_WIDTH = 260;

/**
 * A face captions are set in, chosen the way a system font panel chooses
 * one: a family, from every one installed or carried by the app, found by
 * typing and each named in its own type, and beside it the style, from the
 * weights the family has, upright and, where it has one, italic. The main
 * face and the accent's each have one.
 */
export function TextFontRow({
  font,
  isAccent = false,
  isDisabled,
  onChange,
}: {
  font: TextFont;
  isDisabled: boolean;
  onChange: (font: TextFont) => void;
  isAccent?: boolean;
}) {
  const choices = useFontChoices();
  const fontItems = useMemo<PopupPanelItem[]>(
    () =>
      choices.map((choice) => ({
        font: { family: choice.family },
        id: choice.id,
        label: choice.name,
      })),
    [choices],
  );
  const chosen = chosenFontFamily(choices, font);
  const styles = fontStyles(chosen);
  const weightItems: PopupPanelItem[] = styles.map((style) => ({
    ...(style.detail ? { detail: style.detail } : {}),
    font: { family: chosen.family, ...style.font },
    id: style.id,
    label: style.label,
  }));
  const title = isAccent
    ? t("editor-panels-text-accent")
    : t("editor-panels-text-font");
  const weightLabel = isAccent
    ? t("editor-panels-text-accent-weight")
    : t("editor-panels-text-weight");

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
            className={TEXT_FAMILY_WIDTH}
            id={isAccent ? "caption-accent-font" : "caption-font"}
            isDisabled={isDisabled}
            items={fontItems}
            label={title}
            minimumListWidth={FONT_LIST_WIDTH}
            onSelectionChange={(item) => {
              const family = choices.find((choice) => choice.id === item.id);
              if (family) onChange(fontInFamily(font, family));
            }}
            placeholder={title}
            searchLabel={t("editor-panels-text-font-search")}
            selectedId={chosen.id}
          />
          <PopupSelect
            className={TEXT_WEIGHT_WIDTH}
            id={isAccent ? "caption-accent-weight" : "caption-weight"}
            isDisabled={isDisabled || weightItems.length < 2}
            items={weightItems}
            label={weightLabel}
            onSelectionChange={(item) => {
              const style = styles.find((each) => each.id === item.id);
              if (style) onChange(style.font);
            }}
            placeholder={weightLabel}
            selectedId={fontStyleId(chosen, font)}
          />
        </div>
      )}
    </ControlRow>
  );
}
