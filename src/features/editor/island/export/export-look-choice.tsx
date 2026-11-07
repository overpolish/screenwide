// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { RadioButton, RadioField, RadioGroup } from "react-aria-components";

import { t } from "../../../../i18n/i18n";
import { cn, elementFocusVisible, focusStyles } from "../../../../lib/styling";

import type { ExportLook } from "../../export/export-looks";
import type { ToolPanelExportLook } from "../../tools/tool-panel-export";

const lookName = (look: ExportLook) => {
  switch (look) {
    case "smallest":
      return t("editor-export-look-smallest");
    case "smaller":
      return t("editor-export-look-smaller");
    case "balanced":
      return t("editor-export-look-balanced");
    case "original":
      return t("editor-export-look-original");
  }
};

/** The four looks side by side, smallest first, each a card: its name and
 * the picture's size under it, centred. The chosen card stands on the
 * selected surface. */
export function ExportLookChoice({
  looks,
  onChange,
  value,
}: {
  looks: ToolPanelExportLook[];
  onChange: (look: ExportLook) => void;
  value: ExportLook | null;
}) {
  return (
    <RadioGroup
      aria-label={t("editor-export-quality")}
      // Four equal cards across the island: a long name wraps inside its
      // card rather than widening the island.
      className="grid grid-cols-4 gap-separate"
      onChange={(look) => {
        onChange(look as ExportLook);
      }}
      value={value}
    >
      {looks.map((look) => (
        <RadioField className="flex min-w-0" key={look.id} value={look.id}>
          <RadioButton
            className={cn(
              "flex min-w-0 flex-1 cursor-default flex-col items-center gap-inline rounded-control bg-fill p-inset text-center transition-colors data-[hovered]:not-data-[selected]:bg-fill-strong data-[selected]:bg-selected",
              focusStyles,
              elementFocusVisible,
            )}
          >
            {/* Sized by its card, never sizing it, so a long name wraps
                  where the figures leave room instead of widening the
                  island. */}
            <span className="w-full text-label break-words text-fg contain-inline-size">
              {lookName(look.id)}
            </span>
            <span className="text-caption whitespace-nowrap text-fg-secondary tabular-nums">
              {t("editor-export-dimensions", {
                height: String(look.height),
                width: String(look.width),
              })}
            </span>
          </RadioButton>
        </RadioField>
      ))}
    </RadioGroup>
  );
}
