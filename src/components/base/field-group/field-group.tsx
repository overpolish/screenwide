// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ComponentProps } from "react";

import { cn } from "../../../lib/styling";

import { FieldGroupContext } from "./field-group-context";

export function FieldGroup({ className, ...props }: ComponentProps<"div">) {
  return (
    <FieldGroupContext value>
      <div
        {...props}
        className={cn(
          "field-group has-[[data-control-size=compact]]:rounded-lg rounded-xl bg-neutral transition-colors",
          className,
        )}
      />
    </FieldGroupContext>
  );
}

export function FieldGroupAction({
  className,
  ...props
}: ComponentProps<"div">) {
  return (
    <div
      {...props}
      className={cn("pr-control flex shrink-0 items-center", className)}
      data-field-group-action
    />
  );
}

export function FieldGroupFooter({
  className,
  ...props
}: ComponentProps<"div">) {
  return (
    <div {...props} className={cn("px-control-inset pb-control", className)} />
  );
}
