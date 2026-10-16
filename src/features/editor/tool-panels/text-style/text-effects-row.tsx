// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { PillGroup } from "../../../../components/base/pill-group/pill-group";
import { ControlRow } from "../../../../components/shared/control-row/control-row";
import { t } from "../../../../i18n/i18n";

import { offersOutline } from "./text-templates";

import type { TextStyle } from "../../../../bindings/TextStyle";

type CaptionEffect = "highlightWords" | "outline" | "revealWords";

/**
 * The style's on-or-off options in one row, any of them on together:
 * whether the word being said is highlighted, whether the words are
 * revealed as they are said, and the outline, where the template draws its
 * type without a box. Text that is not spoken offers the outline alone, and
 * no row where there is none. Each segment names its option in full for
 * assistive technology.
 */
export function TextEffectsRow({
  isDisabled,
  onChange,
  spoken = true,
  style,
}: {
  isDisabled: boolean;
  onChange: (style: Partial<TextStyle>) => void;
  style: TextStyle;
  /** Whether the text's words are said, which the word options follow. */
  spoken?: boolean;
}) {
  const effects: { ariaLabel: string; id: CaptionEffect; label: string }[] = [
    ...(spoken ? spokenEffects() : []),
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

/** The options that follow the words as they are said. */
const spokenEffects = (): {
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
