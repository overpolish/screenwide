// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ReactNode, Ref } from "react";

import { cn } from "../../lib/styling";

// A titled window's page: the header, then the content. It paints nothing:
// the native window's material is the surface and the system draws its
// corners. The header and content meet with no gap, as a native title bar
// meets its content; the content brings its own inset.

type WindowShellProps = {
  children: ReactNode;
  className?: string;
  /** Left out by a sheet, which has no title bar of its own. */
  header?: ReactNode;
  /** `content` lets the page be as tall as what it holds, for a window that
   * fits itself to it with `useFitWindowHeight`. */
  height?: "fill" | "content";
  ref?: Ref<HTMLElement>;
};

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
        "flex w-full flex-col overflow-hidden text-fg",
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
