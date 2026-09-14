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
      // hover on the track. Fluent's is a 4px track on the strong fill with
      // an 18px disc knob, a solid fill under an elevation border, carrying
      // an accent dot that grows on hover and shrinks while dragging; its
      // accent fades on hover and press. Colours are the platform tokens and
      // the Fluent geometry is the Windows variant.
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
          className="absolute -inset-x-2.5 top-1/2 h-1.5 -translate-y-1/2 rounded-full bg-control-strong-fill transition-colors group-data-[disabled]:bg-control-strong-fill-disabled windows:h-1"
          style={{ transitionDuration }}
        />
        <SliderFill
          className="absolute top-1/2 -ml-2.5 box-content h-1.5! -translate-y-1/2 rounded-full bg-primary-surface pl-2.5 transition-colors group-data-[disabled]:bg-primary-surface-disabled windows:h-1! windows:group-data-[hovered]:bg-primary-surface-hover windows:group-data-[dragging]:bg-primary-surface-pressed"
          style={{ transitionDuration }}
        />
        <SliderThumb
          className={cn(
            "top-1/2 h-4 w-5 rounded-full bg-control-solid-fill shadow-sm outline-none transition-shadow",
            "group-data-[disabled]:bg-fill-quaternary group-data-[disabled]:shadow-none",
            // A 20px disc (measured), the same width as the macOS knob, so
            // the track's overshoot of half a knob fits both.
            "windows:size-5 windows:shadow-none windows:control-stroke windows:group-data-[disabled]:bg-control-solid-fill",
            // The accent dot: 14px under the pointer, scaled to 10 at rest
            // and 8 while dragging. Scaled rather than resized so it grows
            // about its centre on the compositor, on WinUI's 167ms fast-out
            // slow-in curve.
            "windows:after:absolute windows:after:inset-0 windows:after:m-auto windows:after:size-3.5 windows:after:scale-[0.714] windows:after:rounded-full windows:after:bg-primary-surface windows:after:transition-[scale,background-color] windows:after:duration-[167ms] windows:after:ease-[cubic-bezier(0,0,0,1)] motion-reduce:after:transition-none windows:after:content-['']",
            "windows:data-[hovered]:after:scale-100 windows:data-[hovered]:after:bg-primary-surface-hover",
            "windows:data-[dragging]:after:scale-[0.571] windows:data-[dragging]:after:bg-primary-surface-pressed",
            "windows:group-data-[disabled]:after:bg-primary-surface-disabled",
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
