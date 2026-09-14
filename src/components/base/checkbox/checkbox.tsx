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
  "relative flex size-checkbox shrink-0 transform-gpu items-center justify-center rounded-sm bg-control-alt-fill text-primary-fg inset-ring inset-ring-control-alt-stroke transition-[background-color,box-shadow]",
  "group-data-[hovered]:bg-control-alt-fill-hover group-data-[pressed]:bg-control-alt-fill-pressed group-data-[pressed]:inset-ring-control-alt-stroke-pressed",
  "group-data-[selected]:bg-primary-surface group-data-[selected]:inset-ring-transparent windows:group-data-[selected]:group-data-[hovered]:bg-primary-surface-hover group-data-[selected]:group-data-[pressed]:bg-primary-surface-pressed group-data-[selected]:group-data-[pressed]:text-primary-fg-pressed group-data-[selected]:group-data-[focus-visible]:ring-offset-2 group-data-[selected]:group-data-[focus-visible]:ring-offset-content",
  "group-data-[indeterminate]:bg-primary-surface group-data-[indeterminate]:inset-ring-transparent windows:group-data-[indeterminate]:group-data-[hovered]:bg-primary-surface-hover group-data-[indeterminate]:group-data-[pressed]:bg-primary-surface-pressed group-data-[indeterminate]:group-data-[pressed]:text-primary-fg-pressed",
  "group-data-[disabled]:bg-control-alt-fill-disabled group-data-[disabled]:inset-ring-control-alt-stroke-disabled group-data-[disabled]:text-primary-fg-disabled",
  "group-data-[disabled]:group-data-[selected]:bg-primary-surface-disabled group-data-[disabled]:group-data-[selected]:inset-ring-control-alt-stroke-disabled",
  "group-data-[disabled]:group-data-[indeterminate]:bg-primary-surface-disabled group-data-[disabled]:group-data-[indeterminate]:inset-ring-control-alt-stroke-disabled",
  "[&_svg]:size-icon-mini [&_svg]:stroke-2 [&_svg]:transform-gpu",
  focusStyles,
  groupFocusVisible,
].join(" ");

type CheckboxProps = Omit<AriaCheckboxFieldProps, "children"> & {
  children?: React.ReactNode;
  /** A line under the label explaining the choice, in the secondary tone,
   * as System Settings sets a checkbox's fine print. */
  description?: React.ReactNode;
};

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
                      {isIndeterminate ? <Minus /> : <Check />}
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
