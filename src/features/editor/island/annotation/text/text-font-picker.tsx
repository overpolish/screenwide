// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type CSSProperties, useState } from "react";
import { useFilter } from "react-aria";

import { t } from "../../../../../i18n/i18n";
import { SearchField } from "../../../../../primitives/input-fields/search-field";
import { ListBox } from "../../../../../primitives/listbox/listbox";
import { ListBoxItem } from "../../../../../primitives/listbox-item/listbox-item";
import { PillGroup } from "../../../../../primitives/pill-group/pill-group";
import { revealChosen } from "../../../../../primitives/scroll-area/reveal-chosen";
import { ScrollArea } from "../../../../../primitives/scroll-area/scroll-area";
import {
  chosenFontFamily,
  type FontChoice,
  fontInFamily,
  fontStyleId,
  fontStyles,
} from "../../../tool-panels/text-style/text-fonts";

import { TextPickerLine } from "./text-picker-line";
import { ItalicToggle } from "./text-toggles";

import type { TextFont } from "../../../../../bindings/TextFont";

/** `font` as CSS draws it, so a name can be shown in its own face. */
const faceOf = (family: string, font?: TextFont): CSSProperties => ({
  fontFamily: `"${family}"`,
  ...(font
    ? { fontStyle: font.italic ? "italic" : "normal", fontWeight: font.weight }
    : {}),
});

/** A face a text is set in, folded to a line naming the family and style in
 * that face. `choices` are the families on offer, read once for every
 * line. */
export function TextFontLine({
  choices,
  controls,
  font,
  isAccent = false,
  isOpen,
  onPress,
}: {
  choices: FontChoice[];
  controls: string;
  font: TextFont;
  isOpen: boolean;
  onPress: () => void;
  /** The accent's face rather than the text's. */
  isAccent?: boolean;
}) {
  const chosen = chosenFontFamily(choices, font);
  const styleId = fontStyleId(chosen, font);
  const styleName =
    fontStyles(chosen).find((style) => style.id === styleId)?.label ??
    chosen.name;
  const value = t("editor-island-text-font-value", {
    family: chosen.name,
    style: styleName,
  });
  return (
    <TextPickerLine
      controls={controls}
      isOpen={isOpen}
      label={
        isAccent
          ? t("editor-island-text-accent-font-current", { font: value })
          : t("editor-island-text-font-current", { font: value })
      }
      onPress={onPress}
    >
      <span className="truncate" style={faceOf(chosen.family, font)}>
        {value}
      </span>
    </TextPickerLine>
  );
}

/** The faces on offer, filling the column they open in: a field to find a
 * family by name, the families each written in their own face, scrolling,
 * and under them the chosen family's weights, with whether it is italic
 * beside them where the family has an italic. Choosing a family leaves them
 * open so a weight can follow. */
export function TextFontPanel({
  choices,
  font,
  onChange,
}: {
  choices: FontChoice[];
  font: TextFont;
  onChange: (font: TextFont) => void;
}) {
  const [query, setQuery] = useState("");
  const { contains } = useFilter({ sensitivity: "base" });
  const chosen = chosenFontFamily(choices, font);
  // Its weights, each named upright: the italic is its own choice.
  const upright = { ...chosen, italic: false };
  const weights = fontStyles(upright);
  const italic = chosen.italic && font.italic;
  const shown = choices.filter((choice) => contains(choice.name, query));
  return (
    <div className="flex h-full flex-col gap-inline">
      <SearchField
        aria-label={t("editor-island-text-font-search")}
        onChange={setQuery}
        placeholder={t("editor-island-text-font-search")}
        value={query}
      />
      <ScrollArea onInitialized={revealChosen} rootClassName="min-h-0 flex-1">
        <ListBox
          aria-label={t("editor-island-text-font-families")}
          disallowEmptySelection
          items={shown}
          onSelectionChange={(keys) => {
            if (keys === "all") return;
            const [key] = [...keys];
            const family = choices.find((choice) => choice.id === key);
            if (family) onChange(fontInFamily(font, family));
          }}
          selectedKeys={[chosen.id]}
          selectionMode="single"
        >
          {(choice) => (
            <ListBoxItem id={choice.id} textValue={choice.name}>
              <span style={faceOf(choice.family)}>{choice.name}</span>
            </ListBoxItem>
          )}
        </ListBox>
      </ScrollArea>
      {weights.length > 1 || chosen.italic ? (
        // One line whatever the family offers, so the families above keep
        // their room in a short island; the weights scroll along it.
        <div className="flex items-center gap-separate">
          {weights.length > 1 ? (
            <ScrollArea
              onInitialized={(viewport) => {
                revealChosen(viewport, "[data-selected]", "horizontal");
              }}
              orientation="horizontal"
              rootClassName="h-island-control min-w-0 flex-1"
              scrollbarHidden
            >
              <PillGroup
                aria-label={t("editor-island-text-font-weight")}
                className="w-max"
                items={weights.map((weight) => ({
                  id: weight.id,
                  // A weight named like another is told apart by its number.
                  label: weight.detail
                    ? t("editor-island-text-font-style-detail", {
                        style: weight.label,
                        weight: weight.detail,
                      })
                    : weight.label,
                }))}
                onSelectionChange={(id) => {
                  const weight = weights.find((each) => each.id === id);
                  if (weight) onChange({ ...weight.font, italic });
                }}
                selected={fontStyleId(upright, font)}
              />
            </ScrollArea>
          ) : null}
          {chosen.italic ? (
            <span className="ms-auto shrink-0">
              <ItalicToggle
                italic={italic}
                onChange={(next) => {
                  onChange({ ...font, italic: next });
                }}
              />
            </span>
          ) : null}
        </div>
      ) : null}
    </div>
  );
}
