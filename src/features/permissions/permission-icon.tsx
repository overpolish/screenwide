// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ReactNode } from "react";

// What the permission is for, on a neutral tile the height of a control;
// every row takes the same one, since the Optional badge, not colour, tells
// required from optional.
export function PermissionIcon({ children }: { children: ReactNode }) {
  return (
    <span
      aria-hidden="true"
      className="flex size-control-height items-center justify-center rounded-control bg-fill text-fg-secondary [&_svg]:size-icon"
    >
      {children}
    </span>
  );
}
