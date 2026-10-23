// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { cn, elementFocusVisible, focusStyles } from "../../lib/styling";

/** A painted circle's hairline edge, so white on white or black on black
 * still reads. A colour shown as a mark rather than a swatch takes it too. */
export const swatchEdge = "inset-ring-1 inset-ring-divider";

/** The swatch's sizes: a control's height, or an icon's for a row of many
 * where the control's would crowd. */
export type SwatchSize = "md" | "sm";

// Shared with Color Button's circle.
export const swatchStyles = (isPainted: boolean, size: SwatchSize = "md") =>
  cn(
    "flex shrink-0 cursor-default items-center justify-center overflow-hidden rounded-full outline-offset-2 transition-[background-color,box-shadow,opacity]",
    size === "md"
      ? "size-control-height [&_svg]:size-icon"
      : "size-icon-lg [&_svg]:size-icon-xs",
    "[&_svg]:shrink-0",
    isPainted
      ? cn(swatchEdge, "data-[pressed]:opacity-80")
      : "bg-fill text-fg-secondary data-[hovered]:bg-fill-strong data-[pressed]:bg-fill-strong",
    "data-[disabled]:opacity-40",
    focusStyles,
    elementFocusVisible,
  );
