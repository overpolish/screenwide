// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { MousePointer2 } from "lucide-react";
import { type CSSProperties } from "react";

import { Keyboard } from "../../../primitives/keyboard/keyboard";
import { Desktop } from "../shared/desktop";
import { PanelWindow } from "../shared/panel-window";

import { AREA, WINDOW } from "./recorder-data";

// Choosing Window or Area in the panel hands over to the screen: what is
// under the pointer lights up in the orange, as a selection on the picture
// does, everything else dims, and a hint beside the pointer says what a
// click or a release would choose. Choosing returns to the panel; nothing
// starts recording from here.

export type PointingTarget = "area" | "window";

const HIGHLIGHT: Record<PointingTarget, CSSProperties> = {
  area: { height: "40%", insetInlineStart: "18%", top: "18%", width: "40%" },
  window: { height: "56%", insetInlineStart: "44%", top: "30%", width: "46%" },
};

const POINTER: Record<PointingTarget, CSSProperties> = {
  area: { insetInlineStart: "58%", top: "58%" },
  window: { insetInlineStart: "62%", top: "46%" },
};

function PointingHint({ target }: { target: PointingTarget }) {
  return (
    <PanelWindow className="w-max gap-1 p-inset">
      <span className="flex items-baseline gap-inline">
        <span className="text-heading" dir="auto">
          {target === "window" ? `${WINDOW.app}: ${WINDOW.name}` : "Area"}
        </span>
        <span className="text-caption text-fg-tertiary tabular-nums">
          {target === "window" ? WINDOW.size : AREA.size}
        </span>
      </span>
      <span className="text-body text-fg-secondary">
        {target === "window"
          ? "Click to choose this window."
          : "Release to choose this area."}
      </span>
      <span className="mt-1 flex items-center gap-inline text-caption text-fg-tertiary">
        {target === "window" ? (
          <>
            <Keyboard>Tab</Keyboard>
            steps through windows,
          </>
        ) : null}
        <Keyboard>Escape</Keyboard>
        goes back
      </span>
    </PanelWindow>
  );
}

export function PointingOverlay({ target }: { target: PointingTarget }) {
  return (
    <Desktop className="size-full">
      <span
        className={
          target === "window"
            ? "absolute rounded-[6%/9%] shadow-[0_0_0_999px_rgb(0_0_0/30%)] ring-3 ring-emphasis"
            : "absolute shadow-[0_0_0_999px_rgb(0_0_0/30%)] outline-2 outline-emphasis outline-dashed"
        }
        style={HIGHLIGHT[target]}
      />
      <span className="absolute" style={POINTER[target]}>
        <MousePointer2 className="size-5 fill-black stroke-white" />
        <span className="absolute start-5 top-6">
          <PointingHint target={target} />
        </span>
      </span>
    </Desktop>
  );
}
