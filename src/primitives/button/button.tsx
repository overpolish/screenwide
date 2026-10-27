// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { motion, MotionProps } from "motion/react";
import { Ref } from "react";
import { AriaButtonProps, HoverEvents } from "react-aria";
import { Button as AriaButton } from "react-aria-components";

import { useSkin } from "../skin/skin";

import { buttonVariants } from "./button-variants";

// The primary action is always filled, so a ghost button has no colour.
export type ButtonLook =
  | { color?: "neutral" | "primary"; variant?: "solid" }
  | { variant: "ghost"; color?: "neutral" };

type ButtonProps = AriaButtonProps &
  HoverEvents &
  ButtonLook &
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
  variant,
  ...props
}: ButtonProps) => {
  const skin = useSkin();
  return (
    <MotionAriaButton
      {...props}
      className={buttonVariants({
        className,
        color,
        isDisabled: props.isDisabled,
        skin,
        variant,
      })}
    >
      {children}
    </MotionAriaButton>
  );
};
