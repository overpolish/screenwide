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
        "gap-control px-control-inset py-tight flex flex-row items-center justify-center rounded-xl bg-neutral text-xs",
        className,
      )}
    >
      {children}
    </div>
  );
}
