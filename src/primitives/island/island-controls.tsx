// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AlertCircle, ChevronDown } from "lucide-react";
import { motion } from "motion/react";
import { type ReactNode, type Ref } from "react";

import { motionDurations, motionEasings } from "../../lib/motion";
import { cn } from "../../lib/styling";
import { Text } from "../text/text";

import { IslandFill } from "./island-fill";

// What the island's content is made of. Its controls are circles of one size,
// a size up from a window's; a toggle says so with `aria-pressed`. None has a
// tooltip: the island sits by the hardware, where a label springing up under
// the pointer would be clutter, so each control is drawn to say what it is
// and names itself only to assistive technology.

type ButtonProps = {
  "aria-label": string;
  children: ReactNode;
  "aria-expanded"?: boolean;
  "aria-pressed"?: boolean;
  className?: string;
  isDisabled?: boolean;
  onPress?: () => void;
  ref?: Ref<HTMLButtonElement>;
};

/** A round control on the island's black: plain, with a fill growing in under
 * the pointer, and a stronger one while it is on or what it opens is open.
 * Raised, it is the one control that names a choice rather than switching
 * something. */
export function IslandButton({
  children,
  className,
  isDisabled,
  isOpen = false,
  isRaised = false,
  onPress,
  ref,
  ...aria
}: ButtonProps & { isOpen?: boolean; isRaised?: boolean }) {
  return (
    <button
      {...aria}
      className={cn(
        "group relative isolate flex size-island-control shrink-0 items-center justify-center rounded-full transition-[background-color,color,opacity] disabled:opacity-40",
        isRaised &&
          "overflow-hidden text-fg control-raised hover:control-raised-hover active:control-raised-pressed",
        className,
      )}
      disabled={isDisabled}
      onClick={onPress}
      ref={ref}
      type="button"
    >
      {isRaised ? null : <IslandFill isChosen={isOpen} />}
      {children}
    </button>
  );
}

/** The island's one primary action: a raised pill in the primary colour, a
 * little wider than the circles around it. */
export function IslandPrimaryButton({
  children,
  className,
  isDisabled,
  onPress,
  ref,
  ...aria
}: ButtonProps) {
  return (
    <button
      {...aria}
      className={cn(
        "flex h-island-control w-12 shrink-0 items-center justify-center rounded-full bg-primary text-on-primary primary-raised transition-[background-color,opacity] hover:bg-primary-hover active:bg-primary-pressed active:primary-raised-pressed disabled:opacity-40",
        className,
      )}
      disabled={isDisabled}
      onClick={onPress}
      ref={ref}
      type="button"
    >
      {children}
    </button>
  );
}

/** A control with choices behind it: a small chevron just clear of it opens
 * them, plain on the black, so it reads as belonging to the control before
 * it and not to the one after. */
export function IslandOptions({
  children,
  isDisabled = false,
  isOpen,
  label,
  onOpen,
}: {
  children: ReactNode;
  isOpen: boolean;
  label: string;
  onOpen: () => void;
  isDisabled?: boolean;
}) {
  return (
    <span className="flex items-center">
      {children}
      <button
        aria-expanded={isOpen}
        aria-label={label}
        className={cn(
          "ms-0.5 flex h-island-control w-icon-sm items-center justify-center transition-[color,opacity] disabled:opacity-40",
          isOpen ? "text-fg" : "text-fg-tertiary enabled:hover:text-fg",
        )}
        disabled={isDisabled}
        onClick={onOpen}
        type="button"
      >
        <ChevronDown aria-hidden className="size-3" />
      </button>
    </span>
  );
}

/** A row of a list in the island's section, rounded as every part is. The
 * last row sits in the section's bottom corners, so its own bottom corners
 * follow them, concentric. */
export function IslandListRow({
  children,
  isChosen,
  onPress,
}: {
  children: ReactNode;
  onPress: () => void;
  isChosen?: boolean;
}) {
  return (
    <button
      aria-pressed={isChosen}
      className="flex h-control-height items-center gap-inline rounded-part px-inset-sm text-start text-body last:rounded-b-(--island-list-radius) hover:bg-fill"
      onClick={onPress}
      type="button"
    >
      {children}
    </button>
  );
}

/** A line the island grows when something needs doing, with at most one
 * action that does it. It takes a ref so `popLayout` can lift it out while
 * it fades away. A long message wraps rather than widening the island, when
 * it is given the width of what sits under it. */
export function IslandWarning({
  action,
  className,
  message,
  onAction,
  ref,
}: {
  message: string;
  action?: string;
  className?: string;
  onAction?: () => void;
  ref?: Ref<HTMLDivElement>;
}) {
  return (
    <motion.div
      animate={{ opacity: 1 }}
      className={cn("flex items-center gap-inline", className)}
      exit={{ opacity: 0 }}
      initial={{ opacity: 0 }}
      ref={ref}
      role="status"
      transition={{ duration: motionDurations.state, ease: motionEasings.out }}
    >
      <span className="flex shrink-0 text-error">
        <AlertCircle aria-hidden className="size-icon" />
      </span>
      <Text className="grow" variant="label">
        {message}
      </Text>
      {action ? (
        <button
          className="h-control-height shrink-0 rounded-full bg-fill-strong px-inset text-label hover:bg-fill"
          onClick={onAction}
          type="button"
        >
          {action}
        </button>
      ) : null}
    </motion.div>
  );
}
