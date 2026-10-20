// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Check, Minus } from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import {
  CheckboxButton as AriaCheckboxButton,
  CheckboxField as AriaCheckboxField,
  type CheckboxFieldProps as AriaCheckboxFieldProps,
} from "react-aria-components";

import { cn, focusStyles, groupFocusVisible } from "../../../lib/styling";

// A native checkbox: the inset-control box (16px plain fill on macOS, 20px
// with a strong hairline stroke in Fluent), the accent with a white check
// when selected, a body label. Every state colour is a platform token: on
// macOS hover equals rest and the strokes are transparent; Fluent darkens
// the box on hover and press and fades a checked box on hover. Disabled
// dims both parts. The stroke is an inset ring so it composes with the
// focus ring and adds no size.
const boxStyles = [
  "relative flex size-checkbox shrink-0 transform-gpu items-center justify-center rounded-checkbox bg-control-alt-fill text-primary-fg inset-ring inset-ring-control-alt-stroke transition-[background-color,box-shadow]",
  "group-data-[hovered]:bg-control-alt-fill-hover group-data-[pressed]:bg-control-alt-fill-pressed group-data-[pressed]:inset-ring-control-alt-stroke-pressed",
  "group-data-[selected]:bg-primary-surface group-data-[selected]:inset-ring-transparent windows:group-data-[selected]:group-data-[hovered]:bg-primary-surface-hover group-data-[selected]:group-data-[pressed]:bg-primary-surface-pressed group-data-[selected]:group-data-[pressed]:text-primary-fg-pressed group-data-[selected]:group-data-[focus-visible]:ring-offset-2 group-data-[selected]:group-data-[focus-visible]:ring-offset-content",
  "group-data-[indeterminate]:bg-primary-surface group-data-[indeterminate]:inset-ring-transparent windows:group-data-[indeterminate]:group-data-[hovered]:bg-primary-surface-hover group-data-[indeterminate]:group-data-[pressed]:bg-primary-surface-pressed group-data-[indeterminate]:group-data-[pressed]:text-primary-fg-pressed",
  "group-data-[disabled]:bg-control-alt-fill-disabled group-data-[disabled]:inset-ring-control-alt-stroke-disabled group-data-[disabled]:text-primary-fg-disabled",
  "group-data-[disabled]:group-data-[selected]:bg-primary-surface-disabled group-data-[disabled]:group-data-[selected]:inset-ring-control-alt-stroke-disabled",
  "group-data-[disabled]:group-data-[indeterminate]:bg-primary-surface-disabled group-data-[disabled]:group-data-[indeterminate]:inset-ring-control-alt-stroke-disabled",
  "[&_svg]:transform-gpu [&_svg.lucide]:size-icon-mini [&_svg.lucide]:stroke-2",
  focusStyles,
  groupFocusVisible,
].join(" ");

// AppKit's glyphs, fitted to NSButton's regular checkbox rendered at 4x on
// macOS 27: round-capped strokes about 2pt wide, twice Lucide's at this size.
// Drawn in the box's own 16-unit space, so they sit where AppKit puts them.
// Fluent's lighter glyphs are Lucide's.
const macGlyphs = {
  checked: { d: "M4.05 8.55 6.85 11.95 11.75 4.55", width: 2.05 },
  indeterminate: { d: "M5.8 8h4.45", width: 1.95 },
};

function MacGlyph({ state }: { state: keyof typeof macGlyphs }) {
  const { d, width } = macGlyphs[state];
  return (
    <svg
      aria-hidden
      className="size-full windows:hidden"
      fill="none"
      viewBox="0 0 16 16"
    >
      <path
        d={d}
        stroke="currentColor"
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth={width}
      />
    </svg>
  );
}

type CheckboxProps = Omit<
  AriaCheckboxFieldProps,
  "aria-label" | "aria-labelledby" | "children"
> & {
  description?: React.ReactNode;
} &
  // A box always has a name: its label, or one given for a box drawn alone.
  (
    | {
        children: NonNullable<React.ReactNode>;
        "aria-label"?: string;
        "aria-labelledby"?: string;
      }
    | { "aria-label": string; "aria-labelledby"?: string; children?: never }
    | { "aria-labelledby": string; "aria-label"?: string; children?: never }
  );

export const Checkbox = ({
  children,
  description,
  ...props
}: CheckboxProps) => {
  return (
    <AriaCheckboxField {...props} className="contents">
      {/* With a description the box lines up with the label's first line
          rather than the block's middle. */}
      <AriaCheckboxButton
        className={cn(
          "group inline-flex cursor-default gap-control-inset text-body text-content-fg outline-none data-[disabled]:text-control-fg-disabled",
          description ? "items-start" : "items-center",
        )}
      >
        {({ isIndeterminate, isSelected }) => {
          const state = isIndeterminate
            ? "indeterminate"
            : isSelected
              ? "selected"
              : null;

          return (
            <>
              <span className={boxStyles}>
                <AnimatePresence initial={false}>
                  {state ? (
                    <motion.span
                      animate={{ opacity: 1, scale: 1 }}
                      className="absolute inset-0 flex items-center justify-center"
                      exit={{ opacity: 0, scale: 0 }}
                      initial={{ opacity: 0, scale: 0 }}
                      key={state}
                      transition={{ duration: 0.12, ease: "easeOut" }}
                    >
                      <MacGlyph
                        state={isIndeterminate ? "indeterminate" : "checked"}
                      />
                      {isIndeterminate ? (
                        <Minus className="hidden windows:block" />
                      ) : (
                        <Check className="hidden windows:block" />
                      )}
                    </motion.span>
                  ) : null}
                </AnimatePresence>
              </span>
              {description ? (
                <span className="flex min-w-0 flex-col">
                  <span>{children}</span>
                  <span className="text-subheadline text-content-fg-secondary group-data-[disabled]:text-content-fg-tertiary">
                    {description}
                  </span>
                </span>
              ) : (
                children
              )}
            </>
          );
        }}
      </AriaCheckboxButton>
    </AriaCheckboxField>
  );
};
