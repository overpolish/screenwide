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
import { useState } from "react";

import { IconToggleButton } from "../../../primitives/button/icon-button";

// The eleven annotation tools around the pointer. In the app the compositor
// draws it with the selection's other on-picture controls, so it follows the
// press without a window of its own. Flick toward a tool to take it; hold
// still and the ring shows; each tool keeps its direction, so a learned
// flick needs no ring at all. The tool in hand is in the primary colour, as
// a chosen tool is in every toolbar.
//
// Behind the ring the picture goes out of focus, strongest under the disc
// and fading out past its edge, so the ring sits on the work rather than
// over it. The compositor already blurs the picture on the GPU for a blur
// redaction (`compositor/whole_blur.rs`); the ring would take that blur
// through a soft round mask. A panel window could only blur evenly, through
// its material, with no fall-off, which is why the ring belongs to the
// compositor. Here the page's backdrop filter stands in.

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

const RADIUS = 80;

export function AnnotationRing() {
  const [chosen, setChosen] = useState<string>("Arrow");
  return (
    <div
      className="absolute"
      dir="ltr"
      style={{ insetInlineStart: "38%", top: "44%" }}
    >
      <span className="absolute size-80 -translate-1/2 rounded-full backdrop-blur-md [mask-image:radial-gradient(circle,black_55%,transparent_70%)]" />
      {/* Dark's surface is a heavy grey over a light picture, so it tints
          less there and lets more of the blur through. */}
      <span className="absolute size-52 -translate-1/2 rounded-full bg-surface/60 shadow-lg inset-ring inset-ring-divider dark:bg-surface/30" />
      {TOOLS.map(({ icon: Icon, label }, index) => {
        // Arrow at the top, the rest clockwise.
        const angle = (index / TOOLS.length) * 2 * Math.PI - Math.PI / 2;
        return (
          <span
            className="absolute -translate-1/2"
            key={label}
            style={{
              insetInlineStart: Math.cos(angle) * RADIUS,
              top: Math.sin(angle) * RADIUS,
            }}
          >
            <IconToggleButton
              aria-label={label}
              className="rounded-full"
              isSelected={label === chosen}
              onChange={() => {
                setChosen(label);
              }}
            >
              <Icon />
            </IconToggleButton>
          </span>
        );
      })}
      <span className="absolute -translate-1/2 text-heading">{chosen}</span>
    </div>
  );
}
