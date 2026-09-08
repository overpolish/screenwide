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
          "group inline-flex cursor-default items-center gap-control-inset text-body text-content-fg outline-none data-[disabled]:text-content-fg-tertiary",
          className,
        )}
      >
        {({ isSelected }) => (
          <>
            {children}
            <span
              // The macOS 26 mini switch, the size used in grouped settings
              // rows, measured from a live capture: a 36 by 16 track on the
              // fill when off and the accent when on, with a white 21 by 12
              // capsule knob inset 2px. No hover; a press darkens the track;
              // disabled dims the control.
              className={cn(
                "relative inline-flex h-4 w-9 shrink-0 items-center rounded-full bg-fill p-0.5 transition-colors",
                "group-data-[pressed]:bg-fill-secondary",
                "group-data-[selected]:bg-primary-surface group-data-[selected]:group-data-[pressed]:bg-primary-surface-pressed",
                "group-data-[disabled]:opacity-50",
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
                animate={{ x: isSelected ? 11 : 0 }}
                className="h-3 w-[21px] rounded-full bg-white shadow-sm"
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
