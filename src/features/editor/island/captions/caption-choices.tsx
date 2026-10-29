// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../../i18n/i18n";
import { PillGroup } from "../../../../primitives/pill-group/pill-group";
import { OptionToggle } from "../../../annotations/options/option-toggle";
import { SizeGlyph } from "../annotation/text/text-size-choice";

import {
  AllCapsGlyph,
  FillerGlyph,
  HoldGlyph,
  LinesGlyph,
  ProfanityGlyph,
  PunctuationGlyph,
} from "./caption-glyphs";

import type { CaptionSettings } from "../../../../bindings/CaptionSettings";
import type { TextLines } from "../../../../bindings/TextLines";
import type { TextSize } from "../../../../bindings/TextSize";

// The captions' own choices, beside the ones a text shares with them: how
// large they are from their template's size, how many lines they take, and
// what they say. Each is a choice between looks, drawn rather than named;
// the names are for assistive technology.

/** The steps from the template's size, smallest first; Medium is its own. */
const SIZES: TextSize[] = ["small", "medium", "large", "extra-large"];

const sizeName = (size: TextSize) => {
  switch (size) {
    case "small":
      return t("editor-island-captions-size-small");
    case "medium":
      return t("editor-island-captions-size-medium");
    case "large":
      return t("editor-island-captions-size-large");
    case "extra-large":
      return t("editor-island-captions-size-extra-large");
  }
};

/** How large the type, or its key word (`isAccent`), is drawn, a step from
 * the template's own size: a letter growing, leant for the key word so the
 * two read apart side by side. */
export function CaptionSizeChoice({
  isAccent = false,
  onChange,
  value,
}: {
  onChange: (size: TextSize) => void;
  value: TextSize;
  isAccent?: boolean;
}) {
  return (
    <PillGroup
      aria-label={
        isAccent
          ? t("editor-island-captions-accent-size")
          : t("editor-island-captions-size")
      }
      items={SIZES.map((size, step) => ({
        ariaLabel: sizeName(size),
        icon: <SizeGlyph isItalic={isAccent} step={step} />,
        id: size,
      }))}
      onSelectionChange={(size) => {
        onChange(size as TextSize);
      }}
      selected={value}
    />
  );
}

/** How many lines a caption set in lines may take. */
export function CaptionLinesChoice({
  onChange,
  value,
}: {
  onChange: (lines: TextLines) => void;
  value: TextLines;
}) {
  return (
    <PillGroup
      aria-label={t("editor-island-captions-lines")}
      items={(["one", "two"] as const).map((lines) => ({
        ariaLabel:
          lines === "one"
            ? t("editor-island-captions-one-line")
            : t("editor-island-captions-two-lines"),
        icon: <LinesGlyph lines={lines} />,
        id: lines,
      }))}
      onSelectionChange={(lines) => {
        onChange(lines as TextLines);
      }}
      selected={value}
    />
  );
}

/** Whether the captions are set in capitals whatever was said. */
export function AllCapsToggle({
  allCaps,
  onChange,
}: {
  allCaps: boolean;
  onChange: (allCaps: boolean) => void;
}) {
  return (
    <OptionToggle
      glyph={(isOn) => <AllCapsGlyph isOn={isOn} />}
      isOn={allCaps}
      label={t("editor-island-captions-all-caps")}
      onChange={onChange}
    />
  );
}

/** The rules for what the captions leave out and how long they stay up,
 * whatever the template: each the setting it turns on, its glyph and its
 * name. Profanity is starred out in the style the settings keep. */
const WORD_RULES = {
  fillers: {
    glyph: (isOn: boolean) => <FillerGlyph isOn={isOn} />,
    key: "hideFillers",
    label: () => t("editor-island-captions-hide-fillers"),
  },
  hold: {
    glyph: (isOn: boolean) => <HoldGlyph isOn={isOn} />,
    key: "noGaps",
    label: () => t("editor-island-captions-hold-until-next"),
  },
  profanity: {
    glyph: (isOn: boolean) => <ProfanityGlyph isOn={isOn} />,
    key: "censorProfanity",
    label: () => t("editor-island-captions-censor-profanity"),
  },
  punctuation: {
    glyph: (isOn: boolean) => <PunctuationGlyph isOn={isOn} />,
    key: "removePunctuation",
    label: () => t("editor-island-captions-hide-punctuation"),
  },
} as const;

/** One of those rules, as an animated toggle. */
export function CaptionWordToggle({
  onChange,
  rule,
  settings,
}: {
  onChange: (settings: Partial<CaptionSettings>) => void;
  rule: keyof typeof WORD_RULES;
  settings: CaptionSettings;
}) {
  const { glyph, key, label } = WORD_RULES[rule];
  return (
    <OptionToggle
      glyph={glyph}
      isOn={settings[key]}
      label={label()}
      onChange={(isOn) => {
        onChange({ [key]: isOn });
      }}
    />
  );
}
