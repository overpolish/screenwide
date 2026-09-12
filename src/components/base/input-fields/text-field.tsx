// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

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
  label?: string;
  /** Hint text in the empty field, for a field whose label is its aria name. */
  placeholder?: string;
};

export function TextField({
  className,
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
  } = fieldVariants();

  return (
    <AriaTextField {...props} className={base({ className })}>
      {label ? <Label className={labelSlot()}>{label}</Label> : null}
      <div className={field()}>
        <div className={inputWrapper()}>
          <Input className={input()} placeholder={placeholder} />
        </div>
      </div>
    </AriaTextField>
  );
}
