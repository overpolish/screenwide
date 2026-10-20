// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  ArrowUpRight,
  EyeOff,
  Flashlight,
  Focus,
  Highlighter,
  Image,
  MapPinPlusInside,
  Pencil,
  ScanSearch,
  Square,
  Type,
} from "lucide-react";

import { cn } from "../../../../lib/styling";

// The eleven annotation tools around the pointer. In the app the compositor
// draws it with the selection's other on-picture controls, so it follows the
// press without a window of its own. Flick toward a tool to take it; hold
// still and the ring shows; each tool keeps its direction, so a learned
// flick needs no ring at all.

// The app's annotation glyphs, but for Focus: it keeps a region sharp, which
// a viewfinder says and today's droplet, a blur, does not.
const TOOLS = [
  { icon: ArrowUpRight, label: "Arrow" },
  { icon: Type, label: "Text" },
  { icon: Square, label: "Shape" },
  { icon: Pencil, label: "Draw" },
  { icon: Highlighter, label: "Highlight" },
  { icon: MapPinPlusInside, label: "Counter" },
  { icon: EyeOff, label: "Redact" },
  { icon: Image, label: "Image" },
  { icon: ScanSearch, label: "Magnifier" },
  { icon: Flashlight, label: "Spotlight" },
  { icon: Focus, label: "Focus" },
] as const;

const RADIUS = 84;

export function AnnotationRing({ chosen = "Arrow" }: { chosen?: string }) {
  return (
    <div
      className="absolute"
      dir="ltr"
      style={{ insetInlineStart: "38%", top: "44%" }}
    >
      <span className="combined-card absolute size-[220px] -translate-1/2 rounded-full" />
      {TOOLS.map(({ icon: Icon, label }, index) => {
        // Arrow at the top, the rest clockwise.
        const angle = (index / TOOLS.length) * 2 * Math.PI - Math.PI / 2;
        return (
          <span
            aria-label={label}
            className={cn(
              "absolute flex size-10 -translate-1/2 items-center justify-center rounded-full text-(--fg-2)",
              label === chosen && "combined-accent",
            )}
            key={label}
            style={{
              insetInlineStart: Math.cos(angle) * RADIUS,
              top: Math.sin(angle) * RADIUS,
            }}
          >
            <Icon className="size-[18px]" />
          </span>
        );
      })}
      <span className="absolute -translate-1/2 text-[13px] font-semibold">
        {chosen}
      </span>
    </div>
  );
}
