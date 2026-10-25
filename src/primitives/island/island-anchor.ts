// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useLayoutEffect, useState } from "react";

/**
 * How far along the row, from its start, the middle of `control` is, in px,
 * for a section to hang centred under it; `null` without one. Read after
 * every render, before paint, so the section opens under the control in the
 * frame it is asked for. Both boxes ride the same sideways moves, so their
 * difference is where the control sits in the row whatever the island does.
 */
export function useControlAnchor(
  control: HTMLElement | null | undefined,
  row: HTMLElement | null,
  direction: "ltr" | "rtl",
) {
  const [at, setAt] = useState<number | null>(null);
  // Every render, deliberately, as `useSize` reads sizes: it cannot loop, as
  // an unchanged place keeps the same state and React stops there.
  // eslint-disable-next-line @eslint-react/exhaustive-deps
  useLayoutEffect(() => {
    if (!control || !row) return;
    const controlBox = control.getBoundingClientRect();
    const rowBox = row.getBoundingClientRect();
    const middle = controlBox.left + controlBox.width / 2;
    const next = Math.round(
      direction === "rtl" ? rowBox.right - middle : middle - rowBox.left,
    );
    // eslint-disable-next-line @eslint-react/set-state-in-effect
    setAt(next);
  });
  return control ? at : null;
}
