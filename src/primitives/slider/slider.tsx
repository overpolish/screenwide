// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { use } from "react";
import {
  Slider as AriaSlider,
  SliderFill,
  type SliderProps as AriaSliderProps,
  SliderStateContext,
  SliderThumb,
  SliderTrack,
} from "react-aria-components";

import { cn, elementFocusVisible, focusStyles } from "../../lib/styling";

type SliderProps = Omit<AriaSliderProps<number>, "children" | "className"> & {
  className?: string;
  /** Draw a dot under the track at every step. */
  showTicks?: boolean;
};

// Dots 3px below the track at every step; the dot under the current value is
// hidden, since the knob stands there.
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
        className="absolute top-[calc(50%+6px)] size-0.5 -translate-x-1/2 rounded-full bg-fg-tertiary rtl:translate-x-1/2"
        key={index}
        style={{ insetInlineStart: `${String((index / count) * 100)}%` }}
      />
    ),
  );
}

// The knob on its own, for a slider laid out otherwise than `Slider`, such
// as a divider dragged across a picture. It sits in a React Aria slider's
// track, which places it. A 20 by 16 capsule in the knob material, which
// stands lighter than the track in either appearance.
export function SliderKnob({ className }: { className?: string }) {
  return (
    <SliderThumb
      className={cn(
        "top-1/2 h-mark w-5 rounded-full outline-none control-knob",
        focusStyles,
        elementFocusVisible,
        className,
      )}
    />
  );
}

// The progress bar's track, a 6px capsule on the fill, with the value up to
// the knob in the primary colour, read at a glance as progress is. Disabled,
// the whole slider fades as other controls do.
export function Slider({ className, showTicks, ...props }: SliderProps) {
  const { maxValue = 100, minValue = 0, step = 1 } = props;
  const tickCount = showTicks
    ? Math.max(0, Math.round((maxValue - minValue) / step))
    : 0;

  return (
    <AriaSlider
      {...props}
      className={cn(
        "group flex w-full cursor-default items-center data-[disabled]:opacity-40",
        className,
      )}
    >
      {/* The knob travels inside the track by half its width, so the track
          element is inset by that amount and the visible track and fill
          extend back out to the full width. Ticks share the knob's span. */}
      <SliderTrack
        className={cn("relative mx-2.5 flex-1", tickCount > 0 ? "h-6" : "h-5")}
      >
        <span
          aria-hidden
          className="absolute -inset-x-2.5 top-1/2 h-1.5 -translate-y-1/2 rounded-full bg-fill"
        />
        <SliderFill className="absolute top-1/2 -ms-2.5 box-content h-1.5! -translate-y-1/2 rounded-full bg-primary ps-2.5" />
        <SliderKnob />
        {tickCount > 0 ? (
          <SliderTickMarks count={tickCount} minValue={minValue} step={step} />
        ) : null}
      </SliderTrack>
    </AriaSlider>
  );
}
