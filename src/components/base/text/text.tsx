// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { ComponentPropsWithoutRef } from "react";

import { cn } from "../../../lib/styling";

export type TextProps = ComponentPropsWithoutRef<"p"> & {
  as?: "h1" | "h2" | "h3" | "label" | "p" | "span";
  variant?:
    "body" | "footnote" | "headline" | "section" | "subheadline" | "title";
};

const variants = {
  body: "text-body text-content-fg",
  footnote: "text-footnote text-content-fg-secondary",
  headline: "text-headline text-content-fg",
  section: "text-section text-content-fg-secondary",
  subheadline: "text-subheadline text-content-fg-secondary",
  title: "text-title text-content-fg",
} as const;

/** Typography only: the containing layout provides spacing. */
export function Text({
  as = "p",
  className,
  variant = "body",
  ...props
}: TextProps) {
  // The tag varies, so the props are typed against the paragraph the text
  // normally is; every tag offered takes the same global attributes, so the
  // tag is checked as that paragraph too.
  const Element = as as "p";
  return (
    <Element {...props} className={cn("m-0", variants[variant], className)} />
  );
}
