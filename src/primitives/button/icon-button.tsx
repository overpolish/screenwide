// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { motion, MotionProps } from "motion/react";
import { ReactNode, Ref } from "react";
import { AriaButtonProps, AriaToggleButtonProps } from "react-aria";
import {
  Button as AriaButton,
  ToggleButton as AriaToggleButton,
} from "react-aria-components";

import { cn } from "../../lib/styling";
import { IconSwap } from "../icon-swap/icon-swap";
import { IslandFill } from "../island/island-fill";
import { type Skin, useSkin } from "../skin/skin";

import { ButtonLook } from "./button";
import { buttonVariants } from "./button-variants";

// Button's looks, props and defaults in a square, so an icon button and a
// text button never drift apart. A disabled button that stays focusable says
// so with `aria-disabled`, and fades too. On the island it is a circle of the
// island's size.
const squareOf = (skin: Skin) =>
  cn(
    "px-0 aria-disabled:opacity-40",
    skin === "island" ? "group size-island-control" : "w-control-height",
  );

// A toggle that names itself beside its glyph is a pill as wide as both,
// with the gap between an icon and its label. On the island the pill draws
// no fill until it is hovered or on, so its glyph and words, what is seen of
// it, sit the small inset in from the pill's edge, on the island's text
// column, while the pill's edge keeps to the section's as the fills do.
const pillOf = (skin: Skin) =>
  cn("aria-disabled:opacity-40", skin === "island" && "group px-inset-sm");

type IconButtonProps = AriaButtonProps &
  ButtonLook &
  MotionProps & {
    className?: string;
    ref?: Ref<HTMLButtonElement>;
    slot?: string | null;
  };

type IconToggleButtonProps = Omit<AriaToggleButtonProps, "children"> &
  Omit<MotionProps, "children"> & {
    // A glyph drawn from the state shows that state itself: the button
    // neither fills nor swaps it.
    /** The glyph, or a function of whether it is on. */
    children: ReactNode | ((state: { isSelected: boolean }) => ReactNode);
    className?: string;
    /** Words shown after the glyph, which then name the toggle. */
    label?: ReactNode;
    /** The glyph shown while off. Without one, being on fills the button. */
    off?: ReactNode;
    ref?: Ref<HTMLButtonElement>;
    /** The look while off, as Button's. */
    variant?: "ghost" | "solid";
  };

const MotionAriaButton = motion.create(AriaButton);
const MotionAriaToggleButton = motion.create(AriaToggleButton);

export const IconButton = ({
  children,
  className,
  color,
  variant,
  ...props
}: IconButtonProps) => {
  const skin = useSkin();
  // The island's circles are plain unless asked to stand out: raised for
  // the one that names a choice, primary for the island's action.
  const look =
    variant ?? (skin === "island" && color !== "primary" ? "ghost" : "solid");
  // On the island a plain control fills while what it opens is open.
  const hasIslandFill = skin === "island" && look === "ghost";
  return (
    <MotionAriaButton
      {...props}
      className={cn(
        buttonVariants({
          color,
          isDisabled: props.isDisabled,
          skin,
          variant: look,
        }),
        squareOf(skin),
        className,
      )}
    >
      {hasIslandFill ? (
        <IslandFill isChosen={props["aria-expanded"] === true} />
      ) : null}
      {children}
    </MotionAriaButton>
  );
};

// Off, a toggle is a button in its own look, ghost unless asked otherwise:
// toggles sit in toolbars and beside other controls, where a raised square
// each would crowd the row. A toggle with one glyph is on when it is filled
// in the primary colour, as anything on is; one with a glyph for each state
// says which by the glyph alone and keeps its look. A toggle with a label
// fills while on whatever its glyph does: beside its words a glyph's change
// is too small to read the state by. The two looks set the same background,
// so the look is picked per state rather than layered as variants. On the
// island, on is the island's stronger fill instead, as every island control
// shows what it has on or open.
export const IconToggleButton = ({
  children,
  className,
  label,
  off,
  variant = "ghost",
  ...props
}: IconToggleButtonProps) => {
  const skin = useSkin();
  const isIsland = skin === "island";
  const drawsState = typeof children === "function";
  const fillsWhenOn = label != null || (off == null && !drawsState);
  return (
    <MotionAriaToggleButton
      {...props}
      className={({ isSelected }) =>
        cn(
          buttonVariants(
            fillsWhenOn && isSelected && !isIsland
              ? { color: "primary", isDisabled: props.isDisabled, skin }
              : { isDisabled: props.isDisabled, skin, variant },
          ),
          label == null ? squareOf(skin) : pillOf(skin),
          className,
        )
      }
    >
      {({ isSelected }) => (
        <>
          {isIsland && variant === "ghost" ? (
            <IslandFill isChosen={fillsWhenOn && isSelected} />
          ) : null}
          {typeof children === "function" ? (
            children({ isSelected })
          ) : off == null ? (
            children
          ) : (
            <>
              <span className="invisible flex items-center justify-center">
                {children}
              </span>
              <IconSwap swapKey={isSelected ? "selected" : "deselected"}>
                {isSelected ? children : off}
              </IconSwap>
            </>
          )}
          {label == null ? null : <span>{label}</span>}
        </>
      )}
    </MotionAriaToggleButton>
  );
};

export type { IconButtonProps, IconToggleButtonProps };
