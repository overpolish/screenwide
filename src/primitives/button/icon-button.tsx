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

import { ButtonLook } from "./button";
import { buttonVariants } from "./button-variants";

// Button's looks, props and defaults in a square, so an icon button and a
// text button never drift apart. A disabled button that stays focusable says
// so with `aria-disabled`, and fades too.
const square = "w-control-height px-0 aria-disabled:opacity-40";

type IconButtonProps = AriaButtonProps &
  ButtonLook &
  MotionProps & {
    className?: string;
    ref?: Ref<HTMLButtonElement>;
    slot?: string | null;
  };

type IconToggleButtonProps = AriaToggleButtonProps &
  MotionProps & {
    className?: string;
    /** The glyph shown while off. Without one, being on fills the button. */
    off?: ReactNode;
    ref?: Ref<HTMLButtonElement>;
    /** The look while off, as Button's. */
    variant?: "ghost" | "solid";
  };

const MotionAriaButton = motion.create(AriaButton);
const MotionAriaToggleButton = motion.create(AriaToggleButton);

export const IconButton = ({
  className,
  color,
  variant,
  ...props
}: IconButtonProps) => (
  <MotionAriaButton
    {...props}
    className={cn(
      buttonVariants({ color, isDisabled: props.isDisabled, variant }),
      square,
      className,
    )}
  />
);

// Off, a toggle is a button in its own look, ghost unless asked otherwise:
// toggles sit in toolbars and beside other controls, where a raised square
// each would crowd the row. A toggle with one glyph is on when it is filled
// in the primary colour, as anything on is; one with a glyph for each state
// says which by the glyph alone and keeps its look. The two set the same
// background, so the look is picked per state rather than layered as
// variants.
export const IconToggleButton = ({
  children,
  className,
  off,
  variant = "ghost",
  ...props
}: IconToggleButtonProps) => (
  <MotionAriaToggleButton
    {...props}
    className={({ isSelected }) =>
      cn(
        buttonVariants(
          off == null && isSelected
            ? { color: "primary", isDisabled: props.isDisabled }
            : { isDisabled: props.isDisabled, variant },
        ),
        square,
        className,
      )
    }
  >
    {({ isSelected }) =>
      off == null ? (
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
      )
    }
  </MotionAriaToggleButton>
);

export type { IconButtonProps, IconToggleButtonProps };
