// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type ReactNode } from "react";

import { cn } from "../../../lib/styling";

// Stands in for a native panel window (a tool panel, a menu, a hint). In the
// app the window draws its own material, corner and shadow and the content
// paints nothing; a mock has no window to open, so this frame draws them.
// Only explorations use it.
export function PanelWindow({
  children,
  className,
}: {
  children: ReactNode;
  className?: string;
}) {
  return (
    <div
      className={cn(
        "flex flex-col overflow-hidden rounded-panel bg-surface text-fg shadow-lg inset-ring inset-ring-divider",
        className,
      )}
    >
      {children}
    </div>
  );
}
