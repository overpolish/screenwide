// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { motion, MotionProps } from "motion/react";
import { Ref } from "react";
import { AriaButtonProps } from "react-aria";
import { Button as AriaButton } from "react-aria-components";
import { VariantProps } from "tailwind-variants";

import { buttonVariants } from "./button-variants";

type ButtonProps = AriaButtonProps &
  VariantProps<typeof buttonVariants> &
  MotionProps & {
    className?: string;
    ref?: Ref<HTMLButtonElement>;
    slot?: string;
  };

const MotionAriaButton = motion.create(AriaButton);

export const Button = ({
  children,
  className,
  color,
  size,
  variant,
  ...props
}: ButtonProps) => {
  return (
    <MotionAriaButton
      {...props}
      className={buttonVariants({
        className,
        color,
        isDisabled: props.isDisabled,
        size,
        variant,
      })}
    >
      {children}
    </MotionAriaButton>
  );
};
