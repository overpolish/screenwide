// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { CircleDashed, Cloud, Dices, Sun } from "lucide-react";
import { ReactNode } from "react";
import { TooltipTrigger } from "react-aria-components";

import { t } from "../../../i18n/i18n";
import {
  generatePaletteFromLocked,
  PaletteMode,
} from "../../../lib/palette-generator";
import { IconButton } from "../button/icon-button";
import { Tooltip } from "../tooltip/tooltip";

import { ColorSwatch } from "./color-swatch";

const paletteActions = (): Array<{
  icon: typeof Sun;
  label: string;
  mode: PaletteMode;
}> => [
  { icon: Sun, label: t("controls-palette-bright"), mode: "bright" },
  { icon: Cloud, label: t("controls-palette-dull"), mode: "dull" },
  { icon: CircleDashed, label: t("controls-palette-shades"), mode: "shades" },
  { icon: Dices, label: t("controls-palette-chaotic"), mode: "chaotic" },
];

const swatchKeys = ["one", "two", "three", "four", "five"];

export function ColorPaletteGenerator({
  colors,
  endContent,
  isDisabled,
  locked = colors.map(() => false),
  onChange,
  onLockedChange,
}: {
  colors: string[];
  endContent?: ReactNode;
  isDisabled?: boolean;
  locked?: boolean[];
  onChange?: (colors: string[]) => void;
  onLockedChange?: (locked: boolean[]) => void;
}) {
  return (
    <div className="flex flex-col gap-control-inset">
      <div className="flex items-center justify-between gap-section">
        <span className="text-body text-content-fg">
          {t("controls-palette-colors")}
        </span>
        <div className="flex gap-control">
          {colors.slice(0, swatchKeys.length).map((color, index) => (
            <ColorSwatch
              ariaLabel={t("controls-palette-color", { index: index + 1 })}
              isDisabled={isDisabled}
              isLocked={locked[index]}
              key={swatchKeys[index]}
              onChange={(value) => {
                onChange?.(
                  colors.map((current, colorIndex) =>
                    colorIndex === index ? value : current,
                  ),
                );
              }}
              onContextMenu={(event) => {
                event.preventDefault();
                if (isDisabled) return;
                onLockedChange?.(
                  colors.map((_, colorIndex) =>
                    colorIndex === index
                      ? !locked[index]
                      : (locked[colorIndex] ?? false),
                  ),
                );
              }}
              value={color}
            />
          ))}
        </div>
      </div>
      <div className="flex items-center justify-between gap-section">
        <div className="flex items-center gap-control">
          {paletteActions().map(({ icon: Icon, label, mode }) => (
            <TooltipTrigger delay={400} key={mode}>
              <IconButton
                aria-label={label}
                isDisabled={isDisabled}
                onPress={() => {
                  onChange?.(
                    generatePaletteFromLocked({ colors, locked, mode }),
                  );
                }}
              >
                <Icon />
              </IconButton>
              <Tooltip>{label}</Tooltip>
            </TooltipTrigger>
          ))}
        </div>
        {endContent ?? (
          <span className="text-right text-footnote text-content-fg-secondary">
            {t("controls-palette-lock-hint")}
          </span>
        )}
      </div>
    </div>
  );
}
