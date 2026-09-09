// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AnimatePresence, motion, MotionProps } from "motion/react";
import { ReactNode, Ref } from "react";
import { AriaButtonProps, AriaToggleButtonProps } from "react-aria";
import {
  Button as AriaButton,
  ToggleButton as AriaToggleButton,
} from "react-aria-components";
import { VariantProps } from "tailwind-variants";

import { iconButtonVariants } from "./icon-button-variants";

type IconButtonStyleProps = Omit<
  VariantProps<typeof iconButtonVariants>,
  "hasSelectedBackground" | "isToggle"
> & {
  className?: string;
};

type IconButtonProps = AriaButtonProps &
  IconButtonStyleProps &
  MotionProps & {
    ref?: Ref<HTMLButtonElement>;
    slot?: string | null;
  };

type IconToggleButtonProps = AriaToggleButtonProps &
  Omit<IconButtonStyleProps, "color"> &
  MotionProps & {
    /** Alternate deselected icon. Without one, selection adds a background. */
    off?: ReactNode;
    ref?: Ref<HTMLButtonElement>;
  };

const MotionAriaButton = motion.create(AriaButton);
const MotionAriaToggleButton = motion.create(AriaToggleButton);

export const IconButton = ({
  className,
  color,
  size,
  ...props
}: IconButtonProps) => {
  return (
    <MotionAriaButton
      {...props}
      className={iconButtonVariants({
        className,
        color,
        isDisabled: props.isDisabled,
        size,
      })}
    />
  );
};

export const IconToggleButton = ({
  children,
  className,
  off,
  size,
  ...props
}: IconToggleButtonProps) => {
  const scaleAnimation: MotionProps = {
    animate: { opacity: 1, scale: 1 },
    exit: { opacity: 0, scale: 0 },
    initial: { opacity: 0, scale: 0 },
  };
  const fadeAnimation: MotionProps = {
    animate: { opacity: 1, scale: 1 },
    exit: { opacity: 0, scale: 1 },
    initial: { opacity: 0, scale: 1 },
  };

  return (
    <MotionAriaToggleButton
      {...props}
      className={iconButtonVariants({
        className,
        color: "neutral",
        hasSelectedBackground:
          off == null &&
          !props.isDisabled &&
          props["aria-disabled"] !== true &&
          props["aria-disabled"] !== "true",
        isDisabled: props.isDisabled,
        isToggle: true,
        size,
      })}
    >
      {({ isSelected }) =>
        off == null ? (
          // Same glyph either way: selection is a colour change, which the
          // button's transition already animates.
          children
        ) : (
          <>
            <span className="invisible flex items-center justify-center">
              {children}
            </span>
            <AnimatePresence initial={false}>
              {isSelected ? (
                <motion.span
                  key="selected"
                  {...scaleAnimation}
                  className="absolute inset-0 flex items-center justify-center"
                  exit={fadeAnimation.exit}
                >
                  {children}
                </motion.span>
              ) : (
                <motion.span
                  key="deselected"
                  {...fadeAnimation}
                  className="absolute inset-0 flex items-center justify-center"
                >
                  {off}
                </motion.span>
              )}
            </AnimatePresence>
          </>
        )
      }
    </MotionAriaToggleButton>
  );
};

export type { IconButtonProps, IconToggleButtonProps };
