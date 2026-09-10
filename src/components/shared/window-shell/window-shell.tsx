// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ReactNode, Ref } from "react";

import { cn } from "../../../lib/styling";

type WindowShellProps = {
  children: ReactNode;
  className?: string;
  /** The window's `WindowHeader`. Left out by a sheet, which is placed by the
   * window that opened it and has no title bar of its own. */
  header?: ReactNode;
  /** `fill` takes the window's height, for a window sized in its config;
   * `content` lets the shell be as tall as what it holds, for a window that
   * fits itself to it (see `useFitWindowHeight`). */
  height?: "fill" | "content";
  ref?: Ref<HTMLElement>;
};

/**
 * The surface every titled window is drawn on: the material, the window
 * radius, the label colour, and the header at the top.
 *
 * There is deliberately no gap between the header and the content. The
 * header's own inset is the space below the title, and the content brings
 * its own inset, the way a native window's title bar meets its content.
 * A window must not add one; the header and content would then sit twice
 * the inset apart.
 */
export function WindowShell({
  children,
  className,
  header,
  height = "fill",
  ref,
}: WindowShellProps) {
  return (
    <main
      className={cn(
        "window-surface flex w-full flex-col overflow-hidden rounded-window text-content-fg",
        height === "fill" && "h-full",
        className,
      )}
      ref={ref}
    >
      {header}
      {children}
    </main>
  );
}

export type { WindowShellProps };
