// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useReducedMotion } from "motion/react";
import {
  SwitchButton as AriaSwitchButton,
  SwitchField as AriaSwitchField,
  type SwitchFieldProps as AriaSwitchFieldProps,
} from "react-aria-components";

import { motionDurationCss } from "../../lib/motion";
import { cn, focusStyles, groupFocusVisible } from "../../lib/styling";

// A 36 by 18 track, the knob 2px in. Off, it is the strong fill, a step
// deeper than a slider's track because the knob sits inside it and needs the
// contrast; on, the primary fill, as everything on is, which steps down on
// hover and press as a checked box does. Off takes no hover, as a native
// switch takes none.
const trackStyles = cn(
  "relative inline-flex h-part-height w-9 shrink-0 transform-gpu items-center rounded-full bg-fill-strong p-0.5 transition-[background-color,box-shadow]",
  "group-data-[selected]:bg-primary group-data-[selected]:primary-raised",
  "group-data-[selected]:group-data-[hovered]:bg-primary-hover",
  "group-data-[selected]:group-data-[pressed]:bg-primary-pressed group-data-[selected]:group-data-[pressed]:primary-raised-pressed",
  focusStyles,
  groupFocusVisible,
);

// A 20 by 14 capsule in the knob material, sliding the 12px the track leaves
// it. On the primary fill it takes `on-primary`, which in dark is the warm
// black, so it stands off the near-white track.
const knobStyles = cn(
  // On its own compositor layer for good: a layer that appears only while
  // the slide runs is rasterised at whatever fraction of a pixel it lands on
  // and its edge snaps differently from the track's, which shows as a jagged
  // rim on the knob and the track.
  "h-3.5 w-5 transform-gpu rounded-full backface-hidden transition-[translate] ease-out will-change-transform control-knob",
  "group-data-[selected]:translate-x-3 group-data-[selected]:bg-on-primary rtl:group-data-[selected]:-translate-x-3",
);

type SwitchProps = Omit<AriaSwitchFieldProps, "children" | "className"> & {
  /** The label beside the track. */
  children?: React.ReactNode;
  className?: string;
};

// Disabled, the whole switch fades, label and all, as a checkbox does.
export const Switch = ({ children, className, ...props }: SwitchProps) => {
  const prefersReducedMotion = useReducedMotion();
  const transitionDuration = prefersReducedMotion
    ? "0s"
    : motionDurationCss("state");

  return (
    <AriaSwitchField {...props} className="contents">
      <AriaSwitchButton
        className={cn(
          "group inline-flex cursor-default items-center gap-inline text-body text-fg outline-none data-[disabled]:opacity-40",
          className,
        )}
      >
        {children}
        <span className={trackStyles} style={{ transitionDuration }}>
          <span className={knobStyles} style={{ transitionDuration }} />
        </span>
      </AriaSwitchButton>
    </AriaSwitchField>
  );
};
