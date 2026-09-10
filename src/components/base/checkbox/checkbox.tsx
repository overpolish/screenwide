// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Check, Minus } from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import {
  CheckboxButton as AriaCheckboxButton,
  CheckboxField as AriaCheckboxField,
  type CheckboxFieldProps as AriaCheckboxFieldProps,
} from "react-aria-components";

import { focusStyles, groupFocusVisible } from "../../../lib/styling";

// A native checkbox: a 16px box on the fill ladder, the accent with a white
// bold check when selected, a 13pt label in the label colour. No hover; a
// press stacks the secondary fill behind the box; disabled dims both parts.
const boxStyles = [
  "relative isolate flex size-icon shrink-0 transform-gpu items-center justify-center rounded-sm bg-fill text-primary-fg transition-[background-color,box-shadow]",
  "after:pointer-events-none after:absolute after:inset-0 after:-z-10 after:rounded-[inherit] after:transition-colors after:content-['']",
  "group-data-[pressed]:after:bg-fill-secondary",
  "group-data-[selected]:bg-primary-surface group-data-[selected]:group-data-[pressed]:bg-primary-surface-pressed group-data-[selected]:group-data-[focus-visible]:ring-offset-2 group-data-[selected]:group-data-[focus-visible]:ring-offset-content",
  "group-data-[indeterminate]:bg-primary-surface group-data-[indeterminate]:group-data-[pressed]:bg-primary-surface-pressed",
  "group-data-[disabled]:bg-fill-quaternary group-data-[disabled]:text-content-fg-tertiary",
  "group-data-[disabled]:group-data-[selected]:bg-fill-quaternary group-data-[disabled]:group-data-[indeterminate]:bg-fill-quaternary",
  "[&_svg]:size-icon-mini [&_svg]:stroke-2 [&_svg]:transform-gpu",
  focusStyles,
  groupFocusVisible,
].join(" ");

type CheckboxProps = Omit<AriaCheckboxFieldProps, "children"> & {
  children?: React.ReactNode;
};

export const Checkbox = ({ children, ...props }: CheckboxProps) => {
  return (
    <AriaCheckboxField {...props} className="contents">
      <AriaCheckboxButton className="group inline-flex cursor-default items-center gap-control-inset text-body text-content-fg outline-none data-[disabled]:text-content-fg-tertiary">
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
              {children}
            </>
          );
        }}
      </AriaCheckboxButton>
    </AriaCheckboxField>
  );
};
