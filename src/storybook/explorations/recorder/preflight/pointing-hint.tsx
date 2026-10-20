// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Keyboard } from "../../../../components/base/keyboard/keyboard";
import { AREA, WINDOW } from "../shared/recorder-data";

// Beside the pointer while the panel has stepped aside: what a click or a
// release would choose, and the keys that work without a pointer. Choosing
// returns to the panel; nothing starts recording from here.

export function PointingHint({ target }: { target: "area" | "window" }) {
  return (
    <div className="flex w-max flex-col gap-tight p-control-inset px-section text-content-fg">
      <span className="flex items-baseline gap-control">
        <span className="text-headline" dir="auto">
          {target === "window" ? `${WINDOW.app}: ${WINDOW.name}` : "Area"}
        </span>
        <span className="text-subheadline text-content-fg-secondary tabular-nums">
          {target === "window" ? WINDOW.size : AREA.size}
        </span>
      </span>
      <span className="flex items-center gap-tight text-body text-content-fg-secondary">
        {target === "window"
          ? "Click to choose this window."
          : "Release to choose this area."}
      </span>
      <span className="flex items-center gap-tight text-footnote text-content-fg-tertiary">
        {target === "window" ? (
          <>
            <Keyboard>Tab</Keyboard>
            steps through windows,
          </>
        ) : null}
        <Keyboard>Escape</Keyboard>
        goes back
      </span>
    </div>
  );
}
