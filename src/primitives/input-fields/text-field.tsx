// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ReactNode } from "react";
import {
  Input,
  Label,
  TextField as AriaTextField,
  type TextFieldProps as AriaTextFieldProps,
} from "react-aria-components";

import { fieldVariants } from "../field/field-variants";
import { focusFieldInput } from "../field/focus-field-input";

export type TextFieldProps = Omit<
  AriaTextFieldProps,
  "children" | "className"
> & {
  className?: string;
  /** Decorative glyph before the value; the label still names the field. */
  icon?: ReactNode;
  label?: string;
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
      <div className={field()} onMouseDown={focusFieldInput}>
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
