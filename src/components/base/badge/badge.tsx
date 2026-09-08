// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { clsx } from "clsx";
import { ReactNode } from "react";

type BadgeProps = {
  children: ReactNode;
  className?: string;
};

export function Badge({ children, className }: BadgeProps) {
  return (
    <div
      className={clsx(
        // A native badge: a capsule of subheadline text in the secondary label
        // colour on a faint fill, as in sidebar counts and toolbar item badges.
        "flex flex-row items-center justify-center gap-control rounded-full bg-fill px-control-inset py-tight text-subheadline text-content-fg-secondary [&_svg]:size-icon-mini [&_svg]:shrink-0 [&_svg]:transform-gpu",
        className,
      )}
    >
      {children}
    </div>
  );
}
