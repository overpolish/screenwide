// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { PillGroup } from "../../../../components/base/pill-group/pill-group";
import { ControlRow } from "../../../../components/shared/control-row/control-row";
import { t } from "../../../../i18n/i18n";

import { offersOutline } from "./text-templates";

import type { TextStyle } from "../../../../bindings/TextStyle";

type WordEffect = "highlightWords" | "revealWords";
type CaptionEffect = WordEffect | "outline";

/**
 * The style's on-or-off options in one row, any of them on together:
 * whether the word being said is highlighted, whether the words come on one
 * after another, and the outline, where the template draws its type without
 * a box. `words` names the word options the text offers: a caption both, a
 * text placed by hand that moves the one, a still text neither. No row
 * shows where there is nothing to offer. Each segment names its option in
 * full for assistive technology.
 */
export function TextEffectsRow({
  isDisabled,
  onChange,
  style,
  words = ["highlightWords", "revealWords"],
}: {
  isDisabled: boolean;
  onChange: (style: Partial<TextStyle>) => void;
  style: TextStyle;
  /** The word options offered. */
  words?: WordEffect[];
}) {
  const effects: { ariaLabel: string; id: CaptionEffect; label: string }[] = [
    ...wordEffects().filter(({ id }) => words.some((word) => word === id)),
    ...(offersOutline(style)
      ? [
          {
            ariaLabel: t("editor-panels-text-outline"),
            id: "outline" as const,
            label: t("editor-panels-text-outline"),
          },
        ]
      : []),
  ];
  if (effects.length === 0) return null;
  return (
    <ControlRow title={t("editor-panels-text-effects")}>
      {(controlProps) => (
        <div {...controlProps} role="group">
          <PillGroup
            aria-label={t("editor-panels-text-effects")}
            display="label"
            isDisabled={isDisabled}
            items={effects}
            onSelectionChange={(on) => {
              onChange(
                Object.fromEntries(
                  effects.map(({ id }) => [id, on.includes(id)]),
                ),
              );
            }}
            selected={effects.flatMap(({ id }) => (style[id] ? [id] : []))}
            selectionMode="multiple"
          />
        </div>
      )}
    </ControlRow>
  );
}

/** The options that follow the words. */
const wordEffects = (): {
  ariaLabel: string;
  id: CaptionEffect;
  label: string;
}[] => [
  {
    ariaLabel: t("editor-panels-text-highlight-word"),
    id: "highlightWords",
    label: t("editor-panels-text-effect-highlight"),
  },
  {
    ariaLabel: t("editor-panels-text-reveal-words"),
    id: "revealWords",
    label: t("editor-panels-text-effect-word-by-word"),
  },
];
