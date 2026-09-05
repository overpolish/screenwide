// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";

import type { PressEvent } from "react-aria";

/**
 * Keep consumed keys (capture input or Escape cancellation) from introducing a
 * focus ring into a pointer-started interaction. Focus itself is never removed.
 * Call onKeyDown for every key, marking only keys handled by the interaction as
 * consumed. Navigation keys and blur restore normal focus-visible behaviour.
 */
export function useInteractionFocus() {
  const [pointerStarted, setPointerStarted] = useState(false);

  return {
    className: pointerStarted
      ? "data-[focus-visible]:focus-visible:ring-0! data-[focus-visible]:focus-visible:ring-offset-0!"
      : undefined,
    onBlur: () => {
      setPointerStarted(false);
    },
    onKeyDown: (consumed: boolean) => {
      if (!consumed) setPointerStarted(false);
    },
    onPress: ({ pointerType }: Pick<PressEvent, "pointerType">) => {
      setPointerStarted(
        pointerType === "mouse" ||
          pointerType === "touch" ||
          pointerType === "pen",
      );
    },
  };
}
