// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { motion, useReducedMotion } from "motion/react";
import {
  SwitchButton as AriaSwitchButton,
  SwitchField as AriaSwitchField,
  type SwitchFieldProps as AriaSwitchFieldProps,
} from "react-aria-components";

import {
  motionDurationCss,
  motionDurations,
  motionEasings,
} from "../../../lib/motion";
import { cn, focusStyles, groupFocusVisible } from "../../../lib/styling";

type SwitchProps = Omit<AriaSwitchFieldProps, "children" | "className"> & {
  children?: React.ReactNode;
  className?: string;
};

export const Switch = ({ children, className, ...props }: SwitchProps) => {
  const prefersReducedMotion = useReducedMotion();

  return (
    <AriaSwitchField {...props} className="contents">
      <AriaSwitchButton
        className={cn(
          "group gap-control-inset inline-flex cursor-pointer items-center text-sm text-content-fg outline-none data-[disabled]:cursor-not-allowed data-[disabled]:text-neutral-disabled-fg",
          className,
        )}
      >
        {({ isPressed, isSelected }) => (
          <>
            {children}
            <span
              className={cn(
                "relative inline-flex h-6 w-10 shrink-0 items-center rounded-full bg-neutral p-control transition-colors",
                "group-data-[hovered]:bg-neutral-hover group-data-[pressed]:bg-neutral-pressed",
                "group-data-[selected]:bg-primary-surface group-data-[selected]:group-data-[hovered]:bg-primary-surface-hover group-data-[selected]:group-data-[pressed]:bg-primary-surface-pressed",
                "group-data-[disabled]:bg-neutral-subtle",
                focusStyles,
                groupFocusVisible,
              )}
              style={{
                transitionDuration: prefersReducedMotion
                  ? "0s"
                  : motionDurationCss("state"),
              }}
            >
              <motion.span
                animate={{
                  scale: isPressed ? 0.875 : 1,
                  x: isSelected ? 16 : 0,
                }}
                className="size-4 rounded-full bg-content-fg group-data-[selected]:bg-primary-fg group-data-[disabled]:bg-neutral-disabled-fg"
                initial={false}
                transition={{
                  duration: prefersReducedMotion ? 0 : motionDurations.state,
                  ease: motionEasings.out,
                }}
              />
            </span>
          </>
        )}
      </AriaSwitchButton>
    </AriaSwitchField>
  );
};
