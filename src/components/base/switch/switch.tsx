// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useReducedMotion } from "motion/react";
import {
  SwitchButton as AriaSwitchButton,
  SwitchField as AriaSwitchField,
  type SwitchFieldProps as AriaSwitchFieldProps,
} from "react-aria-components";

import { motionDurationCss } from "../../../lib/motion";
import { cn, focusStyles, groupFocusVisible } from "../../../lib/styling";

// The track. macOS 26's mini switch, the size used in grouped settings rows,
// measured from a live capture: 36 by 16 on the fill when off and the accent
// when on, the knob inset 2px; no hover, a press darkens the track, disabled
// dims the control. Fluent's is 40 by 20 with a strong hairline stroke, its
// off fill recessed, hover and press darkening it, and disabled drawn in its
// own colours rather than dimmed. State colours are the platform tokens; the
// geometry is the Windows variant.
const trackStyles = cn(
  "relative inline-flex h-4 w-9 shrink-0 transform-gpu items-center rounded-full bg-control-alt-fill p-0.5 inset-ring inset-ring-control-alt-stroke transition-[background-color,box-shadow]",
  // Fluent's knob sits 3px inside a 1px stroke; the stroke here is an inset
  // ring that takes no room, so the inset is 4.
  "windows:h-5 windows:w-10 windows:p-1",
  "group-data-[hovered]:bg-control-alt-fill-hover group-data-[pressed]:bg-control-alt-fill-pressed",
  "group-data-[selected]:bg-primary-surface group-data-[selected]:inset-ring-transparent windows:group-data-[selected]:group-data-[hovered]:bg-primary-surface-hover group-data-[selected]:group-data-[pressed]:bg-primary-surface-pressed group-data-[selected]:group-data-[focus-visible]:ring-offset-2 group-data-[selected]:group-data-[focus-visible]:ring-offset-content",
  "group-data-[disabled]:opacity-50 windows:group-data-[disabled]:opacity-100",
  "windows:group-data-[disabled]:bg-control-alt-fill-disabled windows:group-data-[disabled]:inset-ring-control-alt-stroke-disabled",
  "windows:group-data-[disabled]:group-data-[selected]:bg-primary-surface-disabled",
  focusStyles,
  groupFocusVisible,
);

// The knob: a white 21 by 12 capsule on macOS; a 12px dot in Fluent, in the
// secondary label colour when off and the on-accent colour when on, that
// grows to 14 on hover and stretches to 17 by 14 on press. Growth is a
// negative margin, so it spreads from the knob's centre rather than its
// leading edge. The knob slides by the travel less its own width, and the
// margin is folded in, so it lands the same distance from the far inset that
// it rests from the near one whatever its size.
const knobStyles = cn(
  // On its own compositor layer for good, as the icon button is: a layer
  // that appears only while the slide runs is rasterised at whatever
  // fraction of a pixel it lands on and its edge snaps differently from the
  // track's, which shows as a jagged rim on the knob and the track.
  "h-3 w-[21px] transform-gpu backface-hidden will-change-transform rounded-full bg-white shadow-sm [--knob-margin:0px] m-(--knob-margin) transition-[translate,width,height,margin,background-color] ease-out",
  "group-data-[selected]:translate-x-[calc(var(--spacing-switch-travel)-100%-2*var(--knob-margin))]",
  "windows:size-3 windows:bg-content-fg-secondary windows:shadow-none",
  "windows:group-data-[hovered]:size-3.5 windows:group-data-[hovered]:[--knob-margin:-1px]",
  "windows:group-data-[pressed]:h-3.5 windows:group-data-[pressed]:w-[17px] windows:group-data-[pressed]:[--knob-margin:-1px]",
  "windows:group-data-[selected]:bg-primary-fg",
  "windows:group-data-[disabled]:bg-control-fg-disabled",
  "windows:group-data-[disabled]:group-data-[selected]:bg-primary-fg-disabled",
);

type SwitchProps = Omit<AriaSwitchFieldProps, "children" | "className"> & {
  children?: React.ReactNode;
  className?: string;
};

export const Switch = ({ children, className, ...props }: SwitchProps) => {
  const prefersReducedMotion = useReducedMotion();
  const transitionDuration = prefersReducedMotion
    ? "0s"
    : motionDurationCss("state");

  return (
    <AriaSwitchField {...props} className="contents">
      <AriaSwitchButton
        className={cn(
          "group inline-flex cursor-default items-center gap-control-inset text-body text-content-fg outline-none data-[disabled]:text-control-fg-disabled",
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
