// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { PillGroup } from "../../../../components/base/pill-group/pill-group";
import { ControlRow } from "../../../../components/shared/control-row/control-row";
import { t } from "../../../../i18n/i18n";

import { offersOutline } from "./caption-templates";

import type { CaptionStyle } from "../../../../bindings/CaptionStyle";

type CaptionEffect = "highlightWords" | "outline" | "revealWords";

/**
 * The style's on-or-off options in one row, any of them on together:
 * whether the word being said is highlighted, whether the words are
 * revealed as they are said, and the outline, where the template draws its
 * type without a box. Each segment names its option in full for assistive
 * technology.
 */
export function CaptionEffectsRow({
  isDisabled,
  onChange,
  style,
}: {
  isDisabled: boolean;
  onChange: (style: Partial<CaptionStyle>) => void;
  style: CaptionStyle;
}) {
  const effects: { ariaLabel: string; id: CaptionEffect; label: string }[] = [
    {
      ariaLabel: t("editor-panels-captions-highlight-word"),
      id: "highlightWords",
      label: t("editor-panels-captions-effect-highlight"),
    },
    {
      ariaLabel: t("editor-panels-captions-reveal-words"),
      id: "revealWords",
      label: t("editor-panels-captions-effect-word-by-word"),
    },
    ...(offersOutline(style)
      ? [
          {
            ariaLabel: t("editor-panels-captions-outline"),
            id: "outline" as const,
            label: t("editor-panels-captions-outline"),
          },
        ]
      : []),
  ];
  return (
    <ControlRow title={t("editor-panels-captions-effects")}>
      {(controlProps) => (
        <div {...controlProps} role="group">
          <PillGroup
            aria-label={t("editor-panels-captions-effects")}
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
