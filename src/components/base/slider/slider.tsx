// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useReducedMotion } from "motion/react";
import { use } from "react";
import {
  Slider as AriaSlider,
  SliderFill,
  type SliderProps as AriaSliderProps,
  SliderStateContext,
  SliderThumb,
  SliderTrack,
} from "react-aria-components";

import { motionDurationCss } from "../../../lib/motion";
import { cn, elementFocusVisible, focusStyles } from "../../../lib/styling";

type SliderProps = Omit<AriaSliderProps<number>, "children" | "className"> & {
  className?: string;
  /** Draw a dot under the track at every step, as a native stepped slider does. */
  showTicks?: boolean;
};

/**
 * Dots 3px below the track at every step, as a native stepped slider draws
 * them; the dot under the current value is hidden.
 */
function SliderTickMarks({
  count,
  minValue,
  step,
}: {
  count: number;
  minValue: number;
  step: number;
}) {
  const state = use(SliderStateContext);
  const value = state?.values[0];
  const activeIndex =
    value === undefined ? -1 : Math.round((value - minValue) / step);

  return Array.from({ length: count + 1 }, (_, index) =>
    index === activeIndex ? null : (
      <span
        aria-hidden
        className="absolute top-[calc(50%+6px)] size-0.5 -translate-x-1/2 rounded-full bg-content-fg-tertiary"
        key={index}
        style={{ left: `${String((index / count) * 100)}%` }}
      />
    ),
  );
}

export function Slider({ className, showTicks, ...props }: SliderProps) {
  const prefersReducedMotion = useReducedMotion();
  const transitionDuration = prefersReducedMotion
    ? "0s"
    : motionDurationCss("state");
  const { maxValue = 100, minValue = 0, step = 1 } = props;
  const tickCount = showTicks
    ? Math.max(0, Math.round((maxValue - minValue) / step))
    : 0;

  return (
    <AriaSlider
      {...props}
      // A macOS 26 slider without the glass, measured from a live capture: a
      // 6px track on the fill, the accent up to the knob, and a raised capsule
      // knob of 20 by 16 that is white in light and a light grey in dark. No
      // hover on the track.
      className={cn("group flex w-full cursor-default items-center", className)}
    >
      {/* The knob travels inside the track by half its width, so the track
          element is inset by that amount and the visible track and fill
          extend back out to the full width. Ticks share the knob's span. */}
      <SliderTrack
        className={cn(
          "group relative mx-2.5 flex-1",
          tickCount > 0 ? "h-6" : "h-5",
        )}
      >
        <span
          aria-hidden
          className="absolute -inset-x-2.5 top-1/2 h-1.5 -translate-y-1/2 rounded-full bg-fill transition-colors group-data-[disabled]:bg-fill-quaternary"
          style={{ transitionDuration }}
        />
        <SliderFill
          className="absolute top-1/2 -ml-2.5 box-content h-1.5! -translate-y-1/2 rounded-full bg-primary-surface pl-2.5 transition-colors group-data-[disabled]:bg-fill-quaternary"
          style={{ transitionDuration }}
        />
        <SliderThumb
          className={cn(
            "top-1/2 h-4 w-5 rounded-full bg-white shadow-sm outline-none transition-shadow dark:bg-neutral-300",
            "group-data-[disabled]:bg-fill-quaternary group-data-[disabled]:shadow-none",
            focusStyles,
            elementFocusVisible,
          )}
          style={{ transitionDuration }}
        />
        {tickCount > 0 ? (
          <SliderTickMarks count={tickCount} minValue={minValue} step={step} />
        ) : null}
      </SliderTrack>
    </AriaSlider>
  );
}
