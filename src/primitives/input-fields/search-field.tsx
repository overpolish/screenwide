// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Search, X } from "lucide-react";
import { AnimatePresence } from "motion/react";
import {
  Input,
  SearchField as AriaSearchField,
  type SearchFieldProps as AriaSearchFieldProps,
} from "react-aria-components";

import { cn } from "../../lib/styling";
import { IconButton } from "../button/icon-button";
import { fieldVariants } from "../field/field-variants";
import { focusFieldInput } from "../field/focus-field-input";

export type SearchFieldProps = Omit<
  AriaSearchFieldProps,
  "children" | "className"
> & {
  "aria-label": string;
  className?: string;
  placeholder?: string;
};

// The shared field with a magnifier before the text and, once something is
// typed, a clear button at its end. React Aria makes it a search box: Escape
// clears it, Enter submits it, and any button in it is its clear button,
// named for that. The clear button sits in the field's end corner at the
// field's height, a quiet glyph that comes up under the pointer, and takes
// over the end padding while it shows.
export function SearchField({
  className,
  placeholder,
  ...props
}: SearchFieldProps) {
  const { base, field, input, inputWrapper, section } = fieldVariants();

  return (
    <AriaSearchField {...props} className={base({ className })}>
      {({ isDisabled, isEmpty }) => (
        <div className={field()} onMouseDown={focusFieldInput}>
          <div className={inputWrapper({ className: !isEmpty && "pe-0" })}>
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
          <AnimatePresence>
            {isEmpty ? null : (
              <IconButton
                animate={{ opacity: 1 }}
                className={cn(
                  "shrink-0 text-fg-tertiary data-[hovered]:text-fg",
                  "[&_svg.lucide]:size-icon-sm",
                )}
                exit={{ opacity: 0, scale: 0 }}
                initial={{ opacity: 0 }}
                isDisabled={isDisabled}
                variant="ghost"
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
