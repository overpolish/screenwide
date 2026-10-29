// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Search } from "lucide-react";
import {
  Input,
  SearchField as AriaSearchField,
  type SearchFieldProps as AriaSearchFieldProps,
} from "react-aria-components";

import { FieldClearButton } from "../field/field-clear-button";
import { fieldVariants } from "../field/field-variants";
import { focusFieldInput } from "../field/focus-field-input";
import { useSkin } from "../skin/skin";

export type SearchFieldProps = Omit<
  AriaSearchFieldProps,
  "children" | "className"
> & {
  "aria-label": string;
  className?: string;
  placeholder?: string;
};

// The shared field with a magnifier before the text and, once something is
// typed, a clear button at its end (`FieldClearButton`). React Aria makes it
// a search box: Escape clears it, Enter submits it, and any button in it is
// its clear button, named for that.
export function SearchField({
  className,
  placeholder,
  ...props
}: SearchFieldProps) {
  const { base, field, input, inputWrapper, section } = fieldVariants({
    material: useSkin() === "island" ? "island" : "sunken",
  });

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
          <FieldClearButton isDisabled={isDisabled} isShown={!isEmpty} />
        </div>
      )}
    </AriaSearchField>
  );
}
