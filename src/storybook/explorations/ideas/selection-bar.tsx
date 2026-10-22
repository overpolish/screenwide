// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { motion } from "motion/react";

import { cn } from "../../../lib/styling";

import { LookPreview } from "./look-picker";
import { type Look, type LookId, LOOKS, type Size, type Swatch } from "./looks";

// What opens beside a selected mark: the three things people change, and a
// way to the rest. Colours come from the look, so there are three, not a
// palette; size is three steps, not a number; a mark can borrow another
// look. The full panel is one press away, not in the way.

const SIZES: Size[] = ["small", "medium", "large"];

export function SelectionBar({
  look,
  lookOverride,
  onLook,
  onMore,
  onSize,
  onSwatch,
  size,
  swatch,
}: {
  look: Look;
  lookOverride: LookId | null;
  onLook: (id: LookId | null) => void;
  onMore: () => void;
  onSize: (size: Size) => void;
  onSwatch: (swatch: Swatch) => void;
  size: Size;
  swatch: Swatch;
}) {
  const swatches: { color: string; id: Swatch; label: string }[] = [
    { color: "", id: "ink", label: "Auto" },
    { color: look.accent, id: "accent", label: "Accent" },
    { color: look.extra, id: "extra", label: "Extra" },
  ];
  return (
    <motion.div
      animate={{ opacity: 1, scale: 1, y: 0 }}
      className="flex items-center gap-separate rounded-panel bg-[#1c1c1e]/92 px-inset-sm py-1.5 text-white shadow-xl backdrop-blur"
      initial={{ opacity: 0, scale: 0.94, y: 4 }}
      transition={{ bounce: 0, duration: 0.18, type: "spring" }}
    >
      <span className="flex gap-1">
        {swatches.map(({ color, id, label }) => (
          <button
            aria-label={label}
            aria-pressed={swatch === id}
            className={cn(
              "size-6 rounded-full ring-2 ring-offset-2 ring-offset-[#1c1c1e] transition-shadow",
              swatch === id ? "ring-white" : "ring-transparent",
            )}
            key={id}
            onClick={() => {
              onSwatch(id);
            }}
            style={{
              background:
                id === "ink"
                  ? "linear-gradient(135deg, #000 50%, #fff 50%)"
                  : color,
            }}
            title={label}
            type="button"
          />
        ))}
      </span>
      <span className="h-5 w-px bg-white/20" />
      <span className="flex items-center gap-1">
        {SIZES.map((step, index) => (
          <button
            aria-label={step}
            aria-pressed={size === step}
            className={cn(
              "flex size-7 items-center justify-center rounded-part",
              size === step ? "bg-white/20" : "hover:bg-white/10",
            )}
            key={step}
            onClick={() => {
              onSize(step);
            }}
            type="button"
          >
            <span
              className="rounded-full bg-white"
              style={{ height: 4 + index * 3, width: 4 + index * 3 }}
            />
          </button>
        ))}
      </span>
      <span className="h-5 w-px bg-white/20" />
      <span className="flex gap-1">
        {LOOKS.map(({ id, label }) => {
          const isChosen = (lookOverride ?? look.id) === id;
          return (
            <button
              aria-label={label}
              aria-pressed={isChosen}
              className={cn(
                "overflow-hidden rounded-[7px] ring-2 transition-shadow [&_svg]:h-7 [&_svg]:w-12",
                isChosen
                  ? "ring-white"
                  : "ring-transparent hover:ring-white/30",
              )}
              key={id}
              onClick={() => {
                onLook(id === look.id ? null : id);
              }}
              title={label}
              type="button"
            >
              <LookPreview id={id} />
            </button>
          );
        })}
      </span>
      <span className="h-5 w-px bg-white/20" />
      <button
        className="rounded-part px-2 py-1 text-label hover:bg-white/10"
        onClick={onMore}
        type="button"
      >
        More
      </button>
    </motion.div>
  );
}
