// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AnimatePresence, motion, MotionProps } from "motion/react";
import { ReactNode, Ref, use } from "react";
import { AriaButtonProps, AriaToggleButtonProps } from "react-aria";
import {
  Button as AriaButton,
  ToggleButton as AriaToggleButton,
} from "react-aria-components";
import { VariantProps } from "tailwind-variants";

import { elementFocusVisible, focusStyles } from "../../../lib/styling";
import { tv } from "../../../lib/variants";
import { FieldGroupContext } from "../field-group/field-group-context";

const iconButtonVariants = tv({
  base: [
    "relative inline-flex origin-center transform-gpu cursor-pointer items-center justify-center backface-hidden will-change-transform transition select-none",
    "aria-disabled:bg-neutral-subtle aria-disabled:text-neutral-disabled-fg",
    "aria-disabled:data-[selected]:text-neutral-disabled-fg",
    focusStyles,
    elementFocusVisible,
  ],
  compoundVariants: [
    {
      class:
        "text-neutral-disabled-fg data-[disabled]:data-[selected]:text-neutral-disabled-fg",
      isDisabled: true,
      isToggle: true,
    },
    {
      class: "p-0.5 [&_svg]:size-icon-prominent [&_svg]:shrink-0",
      iconSize: "prominent",
      size: "default",
    },
  ],
  defaultVariants: {
    color: "neutral",
    size: "default",
  },
  variants: {
    color: {
      neutral: [
        "bg-transparent text-content-fg",
        "data-[hovered]:bg-neutral",
        "data-[pressed]:bg-neutral-hover",
      ],
      primary: [
        "bg-primary-surface text-primary-fg",
        "data-[hovered]:bg-primary-surface-hover",
        "data-[pressed]:bg-primary-surface-pressed",
      ],
    },
    iconSize: {
      prominent: "",
    },
    isDisabled: {
      true: [
        "cursor-not-allowed! bg-neutral-subtle text-neutral-disabled-fg",
        "data-[disabled]:data-[selected]:text-neutral-disabled-fg",
      ],
    },
    isGrouped: {
      true: ["data-[hovered]:bg-transparent", "data-[pressed]:bg-transparent"],
    },
    isToggle: {
      true: "text-muted data-[selected]:text-content-fg",
    },
    size: {
      compact: "h-6 w-6 rounded-lg p-1 [&_svg]:size-icon-compact",
      default: "h-9 w-9 rounded-xl p-1.5 [&_svg]:size-icon-default",
    },
  },
});

type IconButtonStyleProps = Omit<
  VariantProps<typeof iconButtonVariants>,
  "isGrouped" | "isToggle"
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
    off?: ReactNode;
    ref?: Ref<HTMLButtonElement>;
  };

const MotionAriaButton = motion.create(AriaButton);
const MotionAriaToggleButton = motion.create(AriaToggleButton);

export const IconButton = ({
  className,
  color,
  iconSize,
  size,
  ...props
}: IconButtonProps) => {
  const isGrouped = use(FieldGroupContext);
  return (
    <MotionAriaButton
      {...props}
      className={iconButtonVariants({
        className,
        color,
        iconSize,
        isDisabled: props.isDisabled,
        isGrouped,
        size,
      })}
    />
  );
};

export const IconToggleButton = ({
  children,
  className,
  iconSize,
  off,
  size,
  ...props
}: IconToggleButtonProps) => {
  const isGrouped = use(FieldGroupContext);
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
        iconSize,
        isDisabled: props.isDisabled,
        isGrouped,
        isToggle: true,
        size,
      })}
    >
      {({ isSelected }) => (
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
                {off ?? children}
              </motion.span>
            )}
          </AnimatePresence>
        </>
      )}
    </MotionAriaToggleButton>
  );
};

export type { IconButtonProps, IconToggleButtonProps };
