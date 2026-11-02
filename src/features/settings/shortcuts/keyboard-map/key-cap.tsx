// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  ArrowBigUp,
  ArrowBigUpDash,
  ArrowDown,
  ArrowLeft,
  ArrowRight,
  ArrowRightToLine,
  ArrowUp,
  CircleAlert,
  CornerDownLeft,
  Delete,
  type LucideIcon,
} from "lucide-react";
import { motion, useReducedMotion } from "motion/react";
import { type CSSProperties } from "react";
import { Button, ToggleButton } from "react-aria-components";

import { hotkeyKey } from "../../../../lib/hotkey";
import {
  motionDurationCss,
  motionDurations,
  motionEasings,
} from "../../../../lib/motion";
import { cn, focusStyles, groupFocusVisible } from "../../../../lib/styling";
import { keyLabel } from "../../../../primitives/keyboard/key-label";
import { Keyboard } from "../../../../primitives/keyboard/keyboard";

import type { KeyDef } from "./keyboard-layout";

// One key of the map, read as a key is: its character, or its name, in the
// corner. What it does under the latched modifiers is told by its surface:
// raised with its feature's icon when it does something, flat when free,
// hatched when the system or most apps already use it. A latched modifier
// is filled, as anything on is. The shortcut chosen in the list rings its
// key in the primary ink; one pointed at, in the focus ink. The rings are
// drawn inside the key's edge, beneath its glyphs: a raised key keeps its
// rim and shade around them, its corner icon stays whole, and they never
// reach a neighbour. A key eases between its looks rather than cutting,
// so a latched modifier fills in; a key just given to a shortcut pulses
// its icon once, so the eye finds where the shortcut went.
//
// The key's face sits inside its button, which reaches halfway across the
// gaps round it (`keyboard-map.tsx` sets how far): the gaps are seen, but
// the pointer is over one key or its neighbour wherever it is on the map,
// so the line under the map never blinks back to its hint between two.

export type KeyState = {
  /** What it does, by name, for its accessible name. */
  does: string[];
  isClash: boolean;
  isOff: boolean;
  isPointed: boolean;
  isReserved: boolean;
  isSelected: boolean;
  icon?: LucideIcon;
};

// A hatch in the key's own ink, so it reads in either appearance.
const hatch: CSSProperties = {
  backgroundImage:
    "repeating-linear-gradient(135deg, currentColor 0 1px, transparent 1px 5px)",
};

// Keys both platforms' keyboards print as a symbol, which also fits a key
// too narrow for its name in longer languages. The modifiers other than
// Shift keep the platform's own way, through `Keyboard`.
const PRINTED: Record<string, LucideIcon> = {
  backspace: Delete,
  capslock: ArrowBigUpDash,
  down: ArrowDown,
  enter: CornerDownLeft,
  left: ArrowLeft,
  right: ArrowRight,
  shift: ArrowBigUp,
  tab: ArrowRightToLine,
  up: ArrowUp,
};

const faceClassName = (state: KeyState, isRaised: boolean) =>
  cn(
    "relative flex min-w-0 flex-1 overflow-hidden rounded-part p-inset-sm text-caption leading-none transition-[background-color,box-shadow,color,opacity]",
    isRaised ? "control-raised text-fg" : "bg-fill text-fg-tertiary",
    "group-data-[hovered]:text-fg group-data-[selected]:bg-primary group-data-[selected]:text-on-primary group-data-[selected]:primary-raised",
    state.isOff && "opacity-40",
    state.isPointed && "inset-ring-2 inset-ring-focus",
    state.isSelected && "inset-ring-2 inset-ring-primary",
    focusStyles,
    groupFocusVisible,
  );

const hitClassName = "group flex min-w-0 cursor-default outline-none";

// Grown and back, appearing as it does, on the travel timing a shape uses.
const pulse = {
  animate: { opacity: [0, 1, 1], scale: [1, 1.5, 1] },
  transition: { duration: motionDurations.reshape, ease: motionEasings.out },
};

function KeyFace({
  assigned,
  def,
  state,
}: {
  assigned: number | null;
  def: KeyDef;
  state: KeyState;
}) {
  const Icon = state.icon;
  const Printed = def.name ? PRINTED[def.name] : undefined;
  return (
    <>
      {state.isReserved ? (
        <span
          aria-hidden
          className="absolute inset-0 opacity-25"
          style={hatch}
        />
      ) : null}
      {Printed ? (
        <Printed aria-hidden className="size-icon-sm shrink-0" />
      ) : (
        <Keyboard
          aria-hidden
          className="min-w-0 text-caption text-current"
          variant="plain"
        >
          {def.name ?? hotkeyKey(def.code)}
        </Keyboard>
      )}
      {state.isClash ? (
        <CircleAlert
          aria-hidden
          className="absolute end-0.5 top-0.5 size-icon-xs text-error"
        />
      ) : Icon ? (
        <motion.span
          animate={assigned === null ? undefined : pulse.animate}
          aria-hidden
          className="absolute end-0.5 bottom-0.5 flex"
          // Keyed by the assignment, so each one pulses afresh.
          key={assigned ?? "still"}
          transition={pulse.transition}
        >
          <Icon className="size-icon-xs" />
        </motion.span>
      ) : null}
    </>
  );
}

export function KeyCap({
  assigned = null,
  className,
  def,
  isLatched,
  onHoverChange,
  onPress,
  state,
  style,
}: {
  def: KeyDef;
  isLatched: boolean;
  onHoverChange: (isHovered: boolean) => void;
  onPress: () => void;
  state: KeyState;
  /** Counts the times a shortcut was just given this key, null otherwise. */
  assigned?: number | null;
  /** For the key's face. */
  className?: string;
  /** Where it sits in its row, and how far it reaches into the gaps. */
  style?: CSSProperties;
}) {
  const label = [keyLabel(def.name ?? hotkeyKey(def.code)), ...state.does].join(
    ", ",
  );
  const reduce = useReducedMotion();
  const face = (
    <span
      className={cn(faceClassName(state, state.does.length > 0), className)}
      style={{ transitionDuration: reduce ? "0s" : motionDurationCss("state") }}
    >
      <KeyFace assigned={assigned} def={def} state={state} />
    </span>
  );
  // A modifier is latched by a press, from the keyboard too; the other
  // keys repeat what the list below them does, so they are left out of the
  // tab order.
  return def.modifier ? (
    <ToggleButton
      aria-label={label}
      className={hitClassName}
      isSelected={isLatched}
      onHoverChange={onHoverChange}
      onPress={onPress}
      style={style}
    >
      {face}
    </ToggleButton>
  ) : (
    <Button
      aria-label={label}
      className={hitClassName}
      excludeFromTabOrder
      onHoverChange={onHoverChange}
      onPress={onPress}
      style={style}
    >
      {face}
    </Button>
  );
}
