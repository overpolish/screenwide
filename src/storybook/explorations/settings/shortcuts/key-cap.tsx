// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { CircleAlert } from "lucide-react";
import { motion } from "motion/react";

import { cn, focusStyles } from "../../../../lib/styling";

import { GLYPH, type Reserved, type ShortcutEntry } from "./shortcut-model";

import type { KeyDef } from "./keyboard-layout";

// One key of the map. It reads as a key does: the character at its top
// corner, a named key's word along its bottom, a modifier's glyph above its
// word. What it does under the held modifiers is told by its surface: raised
// with the feature's icon when it does something, flat and faint when free,
// hatched when the system or every app already uses it. Held modifiers are
// filled, as anything on is.

export type KeyState = {
  bound: ShortcutEntry[];
  /** Does something under other modifiers. */
  elsewhere: boolean;
  /** Its shortcut shares the combination with another. */
  hasClash: boolean;
  holds: ShortcutEntry[];
  isDimmed: boolean;
  isHeld: boolean;
  isPressed: boolean;
  isPreviewed: boolean;
  isSelected: boolean;
  reserved?: Reserved;
};

export function KeyCap({
  def,
  height,
  onHover,
  onPress,
  state,
  width,
}: {
  def: KeyDef;
  height: number;
  onHover: (id: string | null) => void;
  onPress: () => void;
  state: KeyState;
  width: number;
}) {
  const { bound, holds, reserved } = state;
  const isBound = bound.length > 0;
  const isOff = isBound && bound.every((entry) => entry.isOff);
  const owner = isBound ? bound[0] : holds.length > 0 ? holds[0] : null;
  const Icon = owner?.icon;
  const isCharacter = def.label.length === 1 && !def.modifier;
  const named = bound.map((entry) => entry.label).join(", ");
  // A modifier's word needs about a key and a quarter at the window's
  // smallest; narrower, the glyph says it alone.
  const hasWord = !def.modifier || width >= 44;

  return (
    <motion.button
      animate={{ scale: state.isPressed ? 0.92 : 1 }}
      aria-label={
        def.modifier
          ? def.label
          : `${def.label || def.id}${named ? `: ${named}` : ""}`
      }
      aria-pressed={def.modifier ? state.isHeld : undefined}
      className={cn(
        "relative shrink-0 cursor-default overflow-hidden rounded-part text-[11px] leading-none outline-none transition-[background-color,box-shadow,opacity,color]",
        focusStyles,
        "focus-visible:ring-2 focus-visible:ring-focus",
        state.isHeld
          ? "bg-primary text-on-primary primary-raised"
          : isBound
            ? "control-raised text-fg"
            : "bg-fill text-fg-tertiary",
        (isOff || state.isDimmed) && "opacity-40",
        state.isPreviewed && !state.isSelected && "ring-2 ring-focus",
        state.isSelected && "ring-2 ring-emphasis",
      )}
      onClick={onPress}
      onPointerEnter={() => {
        onHover(def.id);
      }}
      onPointerLeave={() => {
        onHover(null);
      }}
      style={{ height, width }}
      tabIndex={def.modifier ? 0 : -1}
      transition={{ bounce: 0.4, duration: 0.25, type: "spring" }}
      type="button"
    >
      {reserved && !isBound && !state.isHeld ? (
        <span
          className="absolute inset-0 opacity-25"
          style={{
            backgroundImage:
              "repeating-linear-gradient(135deg, currentColor 0 1px, transparent 1px 5px)",
          }}
        />
      ) : null}
      {def.modifier ? (
        <>
          <span className="absolute end-1.5 top-1.5 text-[12px]">
            {GLYPH[def.modifier]}
          </span>
          {hasWord ? (
            <span className="absolute start-1.5 bottom-1.5 text-[9px]">
              {def.label}
            </span>
          ) : null}
        </>
      ) : isCharacter ? (
        <span className="absolute start-1.5 top-1.5 font-medium">
          {def.label.toUpperCase()}
        </span>
      ) : (
        <span className="absolute start-1.5 bottom-1.5 text-[9px]">
          {def.label}
        </span>
      )}
      {state.hasClash ? (
        <CircleAlert className="absolute end-1 top-1 size-icon-xs text-error" />
      ) : state.elsewhere && !isBound ? (
        <span className="absolute end-1.5 top-1.5 size-1 rounded-full bg-fg-tertiary" />
      ) : null}
      {/* On a modifier the glyph has the top end and the word the bottom
          start, so a held key's icon takes the top start, or the bottom
          start when there is no word. */}
      {Icon ? (
        <Icon
          className={cn(
            "absolute size-icon-xs",
            !def.modifier
              ? "end-1 bottom-1"
              : hasWord
                ? "start-1.5 top-1.5"
                : "start-1.5 bottom-1.5",
          )}
        />
      ) : null}
    </motion.button>
  );
}
