// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { use } from "react";
import {
  Input,
  Label,
  TextField as AriaTextField,
  type TextFieldProps as AriaTextFieldProps,
} from "react-aria-components";

import { cn, focusStyles } from "../../../lib/styling";
import { FieldGroupContext } from "../field-group/field-group-context";

export type TextFieldProps = Omit<
  AriaTextFieldProps,
  "children" | "className"
> & {
  className?: string;
  label?: string;
  size?: "compact" | "default";
};

export function TextField({
  className,
  label,
  size = "default",
  ...props
}: TextFieldProps) {
  const grouped = use(FieldGroupContext);
  return (
    <AriaTextField
      {...props}
      className={cn("group gap-control flex min-w-0 flex-col", className)}
    >
      {label ? (
        <Label
          className={cn(
            "font-medium text-muted",
            size === "compact" ? "text-xs" : "text-sm",
          )}
        >
          {label}
        </Label>
      ) : null}
      <div
        className={cn(
          "relative flex items-center bg-neutral text-content-fg outline-none transition-colors hover:bg-neutral-hover active:bg-neutral-pressed",
          "group-data-[disabled]:cursor-not-allowed group-data-[disabled]:bg-neutral-subtle group-data-[disabled]:text-neutral-disabled-fg",
          focusStyles,
          "has-[input[data-focus-visible]:focus-visible]:ring-1 has-[input[data-focus-visible]:focus-visible]:ring-offset-1",
          "group-data-[invalid]:ring-1 group-data-[invalid]:ring-error",
          size === "compact"
            ? "px-control-inset h-6 rounded-lg"
            : "px-section rounded-xl",
          grouped &&
            "rounded-none bg-transparent hover:bg-transparent active:bg-transparent group-data-[disabled]:bg-transparent",
        )}
      >
        <Input
          className={cn(
            "min-w-0 w-full bg-transparent text-content-fg outline-none focus:selection:bg-accent placeholder:font-extralight placeholder:italic group-data-[disabled]:cursor-not-allowed group-data-[disabled]:text-neutral-disabled-fg",
            size === "compact" ? "text-xs" : "py-control-inset text-sm",
          )}
        />
      </div>
    </AriaTextField>
  );
}
