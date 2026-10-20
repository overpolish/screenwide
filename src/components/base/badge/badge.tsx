// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ReactNode } from "react";

import { cn } from "../../../lib/styling";

type BadgeProps = {
  children: ReactNode;
  className?: string;
};

// A small part: caption text on the faint fill, at a part's corner. Inline
// and a span, so it can sit in a label or a button.
// Tabular figures keep a count or a size from changing width as it changes.
export function Badge({ children, className }: BadgeProps) {
  return (
    <span
      className={cn(
        "inline-flex h-part-height shrink-0 items-center gap-inline rounded-part bg-fill px-inset-sm text-caption whitespace-nowrap text-fg-secondary tabular-nums",
        "[&_svg]:size-icon-sm [&_svg]:shrink-0 [&_svg]:transform-gpu",
        className,
      )}
    >
      {children}
    </span>
  );
}
