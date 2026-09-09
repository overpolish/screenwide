// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ComponentProps, ReactNode } from "react";

import { cn } from "../../../lib/styling";
import { Text } from "../text/text";

type GroupBoxProps = ComponentProps<"section"> & {
  /** A small heading above the box, as a settings pane titles a group. */
  title?: ReactNode;
};

/**
 * The grouped box a settings pane collects rows in: a rounded container on
 * the faintest fill. The box lays its rows out, giving each the same inset
 * so every group in the app reads alike, and keeps a little space above the
 * first and below the last so their fills stay clear of the corners.
 */
export function GroupBox({
  children,
  className,
  title,
  ...props
}: GroupBoxProps) {
  return (
    <section {...props} className={cn("flex flex-col gap-control", className)}>
      {title ? (
        <Text as="h2" className="px-section" variant="section">
          {title}
        </Text>
      ) : null}
      <div className="flex flex-col rounded-control bg-fill-quaternary py-control [&>*]:px-section [&>*]:py-control-inset">
        {children}
      </div>
    </section>
  );
}

export type { GroupBoxProps };
