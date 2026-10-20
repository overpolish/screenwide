// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Check, Minus } from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import {
  CheckboxButton as AriaCheckboxButton,
  CheckboxField as AriaCheckboxField,
  type CheckboxFieldProps as AriaCheckboxFieldProps,
} from "react-aria-components";

import { cn, focusStyles, groupFocusVisible } from "../../lib/styling";

// The box is drawn in the control material when off, as a neutral button
// is, and in the primary fill when on, as everything checked or on is; both
// sink while pressed. The two looks are picked per state rather than layered
// as variants, since both set the same background and neither would win.
const boxStyles = cn(
  "relative flex size-mark shrink-0 transform-gpu items-center justify-center rounded-sm transition-[background-color,box-shadow]",
  "[&_svg]:size-icon-sm [&_svg]:transform-gpu [&_svg]:stroke-[2.5]",
  focusStyles,
  groupFocusVisible,
);
const offStyles =
  "control-raised group-data-[hovered]:control-raised-hover group-data-[pressed]:control-raised-pressed";
const onStyles =
  "bg-primary primary-raised text-on-primary group-data-[hovered]:bg-primary-hover group-data-[pressed]:bg-primary-pressed group-data-[pressed]:primary-raised-pressed";

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
      <AriaCheckboxButton className="group inline-flex cursor-default items-start gap-inline text-body text-fg outline-none data-[disabled]:opacity-40">
        {({ isIndeterminate, isSelected }) => {
          const state = isIndeterminate
            ? "indeterminate"
            : isSelected
              ? "selected"
              : null;

          return (
            <>
              {/* A line tall, so the box sits on the label's first line
                  whether or not a description follows. */}
              <span className="flex h-[1lh] shrink-0 items-center">
                <span className={cn(boxStyles, state ? onStyles : offStyles)}>
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
                        {state === "indeterminate" ? <Minus /> : <Check />}
                      </motion.span>
                    ) : null}
                  </AnimatePresence>
                </span>
              </span>
              {description ? (
                <span className="flex min-w-0 flex-col">
                  <span>{children}</span>
                  <span className="text-caption text-fg-secondary">
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
