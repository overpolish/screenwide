// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ReactNode, useRef } from "react";
import {
  Input,
  Label,
  TextField as AriaTextField,
  type TextFieldProps as AriaTextFieldProps,
} from "react-aria-components";

import { t } from "../../i18n/i18n";
import { FieldClearButton } from "../field/field-clear-button";
import { fieldVariants } from "../field/field-variants";
import { focusFieldInput } from "../field/focus-field-input";
import { useSkin } from "../skin/skin";

export type TextFieldProps = Omit<
  AriaTextFieldProps,
  "children" | "className"
> & {
  className?: string;
  /** Decorative glyph before the value; the label still names the field. */
  icon?: ReactNode;
  /** Ends the field with the clear button a search field has, once
   * something is typed. Needs `value` and `onChange`. */
  isClearable?: boolean;
  label?: string;
  placeholder?: string;
};

export function TextField({
  className,
  icon,
  isClearable = false,
  label,
  placeholder,
  ...props
}: TextFieldProps) {
  const inputRef = useRef<HTMLInputElement>(null);
  const {
    base,
    field,
    input,
    inputWrapper,
    label: labelSlot,
    section,
  } = fieldVariants({
    material: useSkin() === "island" ? "island" : "sunken",
  });
  const showsClear = isClearable && Boolean(props.value);

  return (
    <AriaTextField {...props} className={base({ className })}>
      {label ? <Label className={labelSlot()}>{label}</Label> : null}
      <div className={field()} onMouseDown={focusFieldInput}>
        <div className={inputWrapper({ className: showsClear && "pe-0" })}>
          {icon ? (
            <span aria-hidden className={section()}>
              {icon}
            </span>
          ) : null}
          <Input className={input()} placeholder={placeholder} ref={inputRef} />
        </div>
        {isClearable ? (
          <FieldClearButton
            aria-label={t("controls-field-clear")}
            isDisabled={props.isDisabled ?? false}
            isShown={showsClear}
            onPress={() => {
              props.onChange?.("");
              inputRef.current?.focus();
            }}
          />
        ) : null}
      </div>
    </AriaTextField>
  );
}
