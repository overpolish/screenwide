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
import { compactIconControlStyles } from "../button/icon-button-variants";

type CheckboxProps = Omit<AriaCheckboxFieldProps, "children"> & {
  children?: React.ReactNode;
};

export const Checkbox = ({ children, ...props }: CheckboxProps) => {
  return (
    <AriaCheckboxField {...props} className="contents">
      <AriaCheckboxButton
        className={`group gap-control-inset inline-flex items-center text-sm text-content-fg outline-none data-[disabled]:cursor-not-allowed ${focusStyles}`}
      >
        {({ isIndeterminate, isSelected }) => {
          const state = isIndeterminate
            ? "indeterminate"
            : isSelected
              ? "selected"
              : null;

          return (
            <>
              <span
                className={`relative flex shrink-0 transform-gpu items-center justify-center bg-neutral text-primary-fg transition-[background-color,box-shadow,transform] group-data-[hovered]:bg-neutral-hover group-data-[pressed]:scale-90 group-data-[pressed]:bg-neutral-pressed group-data-[selected]:bg-primary-surface group-data-[selected]:group-data-[hovered]:bg-primary-surface-hover group-data-[selected]:group-data-[pressed]:bg-primary-surface-pressed group-data-[indeterminate]:bg-primary-surface group-data-[indeterminate]:group-data-[hovered]:bg-primary-surface-hover group-data-[indeterminate]:group-data-[pressed]:bg-primary-surface-pressed group-data-[disabled]:bg-neutral-subtle group-data-[disabled]:text-neutral-disabled-fg group-data-[disabled]:group-data-[selected]:bg-neutral-subtle group-data-[disabled]:group-data-[indeterminate]:bg-neutral-subtle ${compactIconControlStyles} ${groupFocusVisible}`}
              >
                <AnimatePresence initial={false}>
                  {state ? (
                    <motion.span
                      animate={{ opacity: 1, scale: 1 }}
                      className="absolute inset-1 flex items-center justify-center"
                      exit={{ opacity: 0, scale: 0 }}
                      initial={{ opacity: 0, scale: 0 }}
                      key={state}
                      transition={{ duration: 0.12, ease: "easeOut" }}
                    >
                      {isIndeterminate ? (
                        <Minus className="transform-gpu" strokeWidth={3} />
                      ) : (
                        <Check className="transform-gpu" strokeWidth={3} />
                      )}
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
