// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useReducedMotion } from "motion/react";
import { RadioButton, RadioField, RadioGroup } from "react-aria-components";

import { t } from "../../../../i18n/i18n";
import { motionDurationCss } from "../../../../lib/motion";
import { cn, elementFocusVisible, focusStyles } from "../../../../lib/styling";
import { formatBytes } from "../../duration";

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

/** The four looks side by side, smallest first, each a card: its name,
 * then the file's size as the figure that tells the looks apart, and the
 * picture's size under it. While a size is being measured the card shows
 * its name and a stub where the figure goes; when it lands the figures
 * fade in, the card keeping its shape. The chosen card stands on the
 * selected surface. Choosing one never measures again; every look is
 * measured once for the edit as it stands. */
export function ExportLookChoice({
  looks,
  onChange,
  value,
}: {
  looks: ToolPanelExportLook[];
  onChange: (look: ExportLook) => void;
  value: ExportLook | null;
}) {
  const fade = useReducedMotion() ? "0s" : motionDurationCss("state");
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
      {looks.map((look) => {
        const isMeasured = look.bytes !== null;
        return (
          <RadioField className="flex min-w-0" key={look.id} value={look.id}>
            <RadioButton
              className={cn(
                "flex min-w-0 flex-1 cursor-default flex-col items-start gap-inline rounded-control bg-fill p-inset text-start transition-colors data-[hovered]:not-data-[selected]:bg-fill-strong data-[selected]:bg-selected",
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
              <span className="flex flex-col whitespace-nowrap">
                <span className="text-body font-medium text-fg tabular-nums">
                  {look.bytes === null ? (
                    <>
                      <span
                        aria-hidden
                        className="inline-block h-2 w-10 rounded-full bg-fill-strong align-middle"
                      />
                      <span className="sr-only">
                        {t("editor-export-measuring")}
                      </span>
                    </>
                  ) : (
                    // Fades in as it first shows, the size having landed.
                    <span
                      className="transition-opacity ease-out starting:opacity-0"
                      style={{ transitionDuration: fade }}
                    >
                      {formatBytes(look.bytes)}
                    </span>
                  )}
                </span>
                {/* Held in place while measuring, unseen and unread, so
                    the card keeps its shape as the figures arrive. */}
                <span
                  aria-hidden={!isMeasured}
                  className={cn(
                    "text-caption text-fg-secondary tabular-nums transition-opacity ease-out",
                    !isMeasured && "opacity-0",
                  )}
                  style={{ transitionDuration: fade }}
                >
                  {t("editor-export-dimensions", {
                    height: String(look.height),
                    width: String(look.width),
                  })}
                </span>
              </span>
            </RadioButton>
          </RadioField>
        );
      })}
    </RadioGroup>
  );
}
