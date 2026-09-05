// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { ComponentPropsWithoutRef } from "react";

import { cn } from "../../../lib/styling";

export type TextProps = ComponentPropsWithoutRef<"p"> & {
  as?: "p" | "span";
  variant?: "body" | "help";
};

const variants = {
  body: "text-sm text-content-fg",
  help: "text-xs text-muted",
} as const;

/** Typography only: the containing layout provides spacing. */
export function Text({
  as: Element = "p",
  className,
  variant = "body",
  ...props
}: TextProps) {
  return (
    <Element
      {...props}
      className={cn("m-0 font-normal", variants[variant], className)}
    />
  );
}
