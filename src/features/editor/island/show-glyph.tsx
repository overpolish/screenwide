// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Eye, EyeOff } from "lucide-react";

import { IconSwap } from "../../../primitives/icon-swap/icon-swap";

/** Lucide's eye while what it stands for shows, and its eye struck through
 * once it is hidden, the one blurring into the other where it sits: the
 * glyph of every island's show or hide toggle. */
export function ShowGlyph({ isShown }: { isShown: boolean }) {
  return (
    <span className="relative size-icon">
      <IconSwap swapKey={isShown ? "shown" : "hidden"}>
        {isShown ? (
          <Eye aria-hidden className="size-icon" />
        ) : (
          <EyeOff aria-hidden className="size-icon" />
        )}
      </IconSwap>
    </span>
  );
}
