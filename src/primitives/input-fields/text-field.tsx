// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ReactNode } from "react";
import {
  Input,
  Label,
  TextField as AriaTextField,
  type TextFieldProps as AriaTextFieldProps,
} from "react-aria-components";

import { fieldVariants } from "./input-field";

export type TextFieldProps = Omit<
  AriaTextFieldProps,
  "children" | "className"
> & {
  className?: string;
  /** A glyph before the text, as the search field's magnifier sits, saying
   * what the field is for. Decorative: the label or aria name names it. */
  icon?: ReactNode;
  label?: string;
  /** Hint text in the empty field, for a field whose label is its aria name. */
  placeholder?: string;
};

export function TextField({
  className,
  icon,
  label,
  placeholder,
  ...props
}: TextFieldProps) {
  const {
    base,
    field,
    input,
    inputWrapper,
    label: labelSlot,
    section,
  } = fieldVariants();

  return (
    <AriaTextField {...props} className={base({ className })}>
      {label ? <Label className={labelSlot()}>{label}</Label> : null}
      <div className={field()}>
        <div className={inputWrapper()}>
          {icon ? (
            <span aria-hidden className={section()}>
              {icon}
            </span>
          ) : null}
          <Input className={input()} placeholder={placeholder} />
        </div>
      </div>
    </AriaTextField>
  );
}
