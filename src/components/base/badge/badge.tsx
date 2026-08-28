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
        "flex flex-row items-center justify-center gap-1 rounded-xl bg-neutral px-2 py-0.5 text-xs",
        className,
      )}
    >
      {children}
    </div>
  );
}
