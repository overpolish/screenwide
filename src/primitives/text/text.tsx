// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { ComponentPropsWithoutRef } from "react";

import { cn } from "../../lib/styling";

type TextProps = ComponentPropsWithoutRef<"p"> & {
  as?: "div" | "h1" | "h2" | "h3" | "label" | "p" | "span";
  /** The text's role, which sets its type and colour together. */
  variant?: "body" | "caption" | "heading" | "label" | "title";
};

// Copy takes its look from its role, so the same role reads the same
// everywhere: a caption is a note about something (a row's description, a
// group's note, a footnote), set back from what it describes. The layout
// around it provides the spacing.
const variants = {
  body: "text-body text-fg",
  caption: "text-caption text-fg-tertiary",
  heading: "text-heading text-fg",
  label: "text-label text-fg",
  title: "text-title text-fg",
} as const;

export function Text({
  as = "p",
  className,
  variant = "body",
  ...props
}: TextProps) {
  // The tag varies, so the props are typed against the paragraph the text
  // normally is; every tag offered takes the same global attributes.
  const Element = as as "p";
  return <Element {...props} className={cn(variants[variant], className)} />;
}
