// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { cn, elementFocusVisible, focusStyles } from "../../lib/styling";

// Shared with Color Button's square. A painted square keeps a hairline edge
// so white on white or black on black still reads.
export const swatchStyles = (isPainted: boolean) =>
  cn(
    "flex size-control-height shrink-0 cursor-default items-center justify-center overflow-hidden rounded-control outline-offset-2 transition-[background-color,box-shadow,opacity]",
    "[&_svg]:size-icon [&_svg]:shrink-0",
    isPainted
      ? "inset-ring-1 inset-ring-divider data-[pressed]:opacity-80"
      : "bg-fill text-fg-secondary data-[hovered]:bg-fill-strong data-[pressed]:bg-fill-strong",
    "data-[disabled]:opacity-40",
    focusStyles,
    elementFocusVisible,
  );
