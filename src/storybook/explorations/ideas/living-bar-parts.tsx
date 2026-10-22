// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type LucideIcon } from "lucide-react";
import { motion } from "motion/react";

import { cn } from "../../../lib/styling";

import { useSimulatedLevel } from "./use-simulated-level";

// The pieces of the living bar: inputs whose glyphs are their own meters, the
// recording dot that breathes with the voice, and the tape of the recording
// so far.

/** An input's glyph, filling from the bottom with its level; off, it is
 * struck through and still. */
export function GlyphMeter({
  icon: Icon,
  isOn,
  isSilent = false,
  label,
  onToggle,
  seed,
}: {
  icon: LucideIcon;
  isOn: boolean;
  label: string;
  onToggle: () => void;
  seed: number;
  isSilent?: boolean;
}) {
  const level = useSimulatedLevel(isSilent || !isOn, seed);
  return (
    <button
      aria-label={label}
      aria-pressed={isOn}
      className="relative flex size-9 items-center justify-center rounded-control hover:bg-white/10"
      onClick={onToggle}
      title={label}
      type="button"
    >
      <Icon
        className={cn("size-5", isOn ? "text-white/35" : "text-white/25")}
      />
      {isOn ? (
        <span
          className="absolute inset-0 flex items-center justify-center"
          style={{
            clipPath: `inset(${String(100 - (18 + level * 82))}% 0 0 0)`,
          }}
        >
          <Icon className="size-5 text-white" />
        </span>
      ) : (
        <span className="absolute h-0.5 w-6 rotate-[-40deg] rounded-full bg-white/60" />
      )}
    </button>
  );
}

/** The recording dot: it swells a little with the voice, and goes grey when
 * paused. */
export function BreathingDot({
  isPaused,
  isSilent,
}: {
  isPaused: boolean;
  isSilent: boolean;
}) {
  const level = useSimulatedLevel(isSilent || isPaused, 1);
  return (
    <span className="relative flex size-4 items-center justify-center">
      <span
        className={cn(
          "absolute rounded-full",
          isPaused ? "bg-white/30" : "bg-error/35",
        )}
        style={{ height: 8 + level * 8, width: 8 + level * 8 }}
      />
      <span
        className={cn(
          "size-2 rounded-full",
          isPaused ? "bg-white/60" : "bg-error",
        )}
      />
    </span>
  );
}

/** The recording so far, with its moments, shown while the bar is open. */
export function Tape({
  moments,
  seconds,
}: {
  moments: number[];
  seconds: number;
}) {
  const span = Math.max(seconds, 10);
  return (
    <div className="relative h-1.5 w-full rounded-full bg-white/15" dir="ltr">
      <span
        className="absolute inset-y-0 start-0 rounded-full bg-white/50"
        style={{ width: `${String((seconds / span) * 100)}%` }}
      />
      {moments.map((at) => (
        <motion.span
          animate={{ opacity: 1, scale: 1, y: 0 }}
          className="absolute -top-2.5 h-2.5 w-0.5 rounded-full bg-[#ffcc00]"
          initial={{ opacity: 0, scale: 0.4, y: 18 }}
          key={at}
          style={{ insetInlineStart: `${String((at / span) * 100)}%` }}
          transition={{ bounce: 0.4, duration: 0.45, type: "spring" }}
        />
      ))}
    </div>
  );
}
