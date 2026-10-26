// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AnimatePresence, motion, useReducedMotion } from "motion/react";
import { type ReactNode, useState } from "react";

import { motionDurations, motionEasings } from "../../../lib/motion";
import { cn } from "../../../lib/styling";
import { Text } from "../../../primitives/text/text";

/** A line of what is going on, its glyph in a slot the size of a control so
 * it stands as far from the island's edge as a control would. A long line,
 * such as several failures one to a line, wraps rather than widening the
 * island past a reading width. */
export function IslandStatus({
  children,
  glyph,
  isError = false,
  trailing,
}: {
  children: ReactNode;
  glyph: ReactNode;
  isError?: boolean;
  /** A figure after the line, such as how far a copy has got. */
  trailing?: ReactNode;
}) {
  return (
    <span className="flex min-w-0 items-center gap-1 pe-2">
      <span
        className={cn(
          "flex size-island-control shrink-0 items-center justify-center",
          isError ? "text-error" : "text-fg-secondary",
        )}
      >
        {glyph}
      </span>
      <Text
        as="span"
        className="w-max max-w-80 py-1 whitespace-pre-line"
        role={isError ? "alert" : "status"}
        variant="label"
      >
        {children}
      </Text>
      {trailing ? (
        <Text
          as="span"
          className="ms-1 text-fg-tertiary tabular-nums"
          variant="label"
        >
          {trailing}
        </Text>
      ) : null}
    </span>
  );
}

/** A glyph ringed by the time left to act on it, draining as it runs out,
 * as a delete's Undo does. Restarts when `runKey` changes. */
export function DrainRing({
  children,
  durationMs,
  runKey,
}: {
  children: ReactNode;
  durationMs: number;
  runKey: string;
}) {
  const reducedMotion = useReducedMotion() ?? false;
  return (
    <span className="relative flex size-6 items-center justify-center">
      <svg
        aria-hidden
        className="absolute inset-0 size-full -rotate-90 fill-none"
        viewBox="0 0 24 24"
      >
        <circle className="stroke-fill-strong" cx="12" cy="12" r="11" />
        {reducedMotion ? null : (
          <motion.circle
            animate={{ pathLength: 0 }}
            className="stroke-fg-secondary [stroke-linecap:round]"
            cx="12"
            cy="12"
            initial={{ pathLength: 1 }}
            key={runKey}
            r="11"
            strokeWidth={1.5}
            transition={{ duration: durationMs / 1_000, ease: "linear" }}
          />
        )}
      </svg>
      <span className="[&_svg]:size-icon-sm">{children}</span>
    </span>
  );
}

/** A count that rolls to its new value: up as it grows, down as it shrinks,
 * like a counter's wheel, so a change catches the eye without a jump. */
export function RollingText({
  children,
  value,
}: {
  children: ReactNode;
  value: number;
}) {
  const reducedMotion = useReducedMotion() ?? false;
  // The last value, kept from render to render to tell which way to roll.
  const [previous, setPrevious] = useState(value);
  const [direction, setDirection] = useState(1);
  if (previous !== value) {
    setDirection(value > previous ? 1 : -1);
    setPrevious(value);
  }
  const offset = reducedMotion ? 0 : 10 * direction;
  return (
    <span className="relative inline-grid overflow-hidden">
      {/* `custom` reaches the value leaving too, so it rolls out the same
          way the new one rolls in. */}
      <AnimatePresence custom={offset} initial={false} mode="popLayout">
        <motion.span
          animate="shown"
          className="col-start-1 row-start-1 whitespace-nowrap"
          custom={offset}
          exit="leaving"
          initial="coming"
          key={value}
          transition={{
            duration: motionDurations.travel,
            ease: motionEasings.out,
          }}
          variants={rolling}
        >
          {children}
        </motion.span>
      </AnimatePresence>
    </span>
  );
}

const rolling = {
  coming: (offset: number) => ({ opacity: 0, y: offset }),
  leaving: (offset: number) => ({ opacity: 0, y: -offset }),
  shown: { opacity: 1, y: 0 },
};
