// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useReducedMotion } from "motion/react";
import {
  Slider as AriaSlider,
  SliderFill,
  type SliderProps as AriaSliderProps,
  SliderThumb,
  SliderTrack,
} from "react-aria-components";

import { motionDurationCss } from "../../../lib/motion";
import { cn, elementFocusVisible, focusStyles } from "../../../lib/styling";

type SliderProps = Omit<AriaSliderProps<number>, "children" | "className"> & {
  className?: string;
};

export function Slider({ className, ...props }: SliderProps) {
  const prefersReducedMotion = useReducedMotion();
  const transitionDuration = prefersReducedMotion
    ? "0s"
    : motionDurationCss("state");

  return (
    <AriaSlider
      {...props}
      className={cn(
        "group flex w-full items-center data-[disabled]:cursor-not-allowed",
        className,
      )}
    >
      <SliderTrack className="group relative h-6 w-full cursor-pointer group-data-[disabled]:cursor-not-allowed">
        <span
          aria-hidden
          className="absolute inset-x-0 top-1/2 h-1 -translate-y-1/2 rounded-full bg-neutral transition-colors group-data-[hovered]:bg-neutral-hover group-active:bg-neutral-pressed group-data-[disabled]:bg-neutral-subtle!"
          style={{ transitionDuration }}
        />
        <SliderFill
          className="absolute top-1/2 h-1! -translate-y-1/2 rounded-full bg-primary-surface transition-colors group-data-[hovered]:bg-primary-surface-hover group-active:bg-primary-surface-pressed group-data-[disabled]:bg-neutral-subtle!"
          style={{ transitionDuration }}
        />
        <SliderThumb
          className={cn(
            "top-1/2 size-4 rounded-full bg-content-fg shadow-sm outline-none transition-shadow",
            "group-data-[disabled]:cursor-not-allowed group-data-[disabled]:bg-neutral-disabled-fg",
            focusStyles,
            elementFocusVisible,
          )}
          style={{ transitionDuration }}
        />
      </SliderTrack>
    </AriaSlider>
  );
}
