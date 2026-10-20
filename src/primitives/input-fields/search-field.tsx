// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Search, X } from "lucide-react";
import { AnimatePresence } from "motion/react";
import {
  Input,
  SearchField as AriaSearchField,
  type SearchFieldProps as AriaSearchFieldProps,
} from "react-aria-components";

import { IconButton } from "../button/icon-button";

import { fieldVariants } from "./input-field";

export type SearchFieldProps = Omit<
  AriaSearchFieldProps,
  "children" | "className"
> & {
  "aria-label": string;
  className?: string;
  placeholder?: string;
};

/**
 * A toolbar search box, as Finder and File Explorer draw one: the bezel of a
 * text field with a magnifier before the text and, once something is typed,
 * a clear button at the trailing edge, drawn and animated as a Select's is.
 * Escape clears it too.
 */
export function SearchField({
  className,
  placeholder,
  ...props
}: SearchFieldProps) {
  const { base, field, input, inputWrapper, section } = fieldVariants();

  return (
    <AriaSearchField {...props} className={base({ className })}>
      {({ isDisabled, isEmpty }) => (
        <div className={field()}>
          <div className={inputWrapper()}>
            <span className={section()}>
              <Search aria-hidden />
            </span>
            <Input
              className={input({
                className: "[&::-webkit-search-cancel-button]:hidden",
              })}
              placeholder={placeholder}
            />
          </div>
          {/* React Aria makes any button in the field its clear button and
              names it. */}
          <AnimatePresence>
            {isEmpty ? null : (
              <IconButton
                animate={{ opacity: 1 }}
                exit={{ opacity: 0, scale: 0 }}
                initial={{ opacity: 0 }}
                isDisabled={isDisabled}
              >
                <X />
              </IconButton>
            )}
          </AnimatePresence>
        </div>
      )}
    </AriaSearchField>
  );
}
