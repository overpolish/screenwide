// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { MousePointer2 } from "lucide-react";
import { type CSSProperties } from "react";

import { Desktop } from "../shared/recorder-parts";

import { PointingHint } from "./pointing-hint";

// Choosing Window or Area in the panel hands over to the screen: what is
// under the pointer lights up, everything else dims, and the hint beside the
// pointer says what a click or a release would choose.

export type PointingTarget = "area" | "window";

const HIGHLIGHT: Record<PointingTarget, CSSProperties> = {
  area: { height: "40%", insetInlineStart: "18%", top: "18%", width: "40%" },
  window: { height: "56%", insetInlineStart: "44%", top: "30%", width: "46%" },
};

const POINTER: Record<PointingTarget, CSSProperties> = {
  area: { insetInlineStart: "58%", top: "58%" },
  window: { insetInlineStart: "62%", top: "46%" },
};

export function PointingOverlay({ target }: { target: PointingTarget }) {
  return (
    <Desktop className="size-full">
      <span
        className={
          target === "window"
            ? "absolute rounded-[6%/9%] bg-primary/10 shadow-[0_0_0_999px_rgb(0_0_0/30%)] ring-3 ring-primary"
            : "absolute outline-2 outline-white outline-dashed shadow-[0_0_0_999px_rgb(0_0_0/30%)]"
        }
        style={HIGHLIGHT[target]}
      />
      <span className="absolute" style={POINTER[target]}>
        <MousePointer2 className="size-5 fill-black stroke-white" />
        {/* Stands in for the hint window's own material. */}
        <span className="absolute start-5 top-6 rounded-window bg-content/85 shadow-xl backdrop-blur-xl">
          <PointingHint target={target} />
        </span>
      </span>
    </Desktop>
  );
}
