// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ReactNode } from "react";

import { TimelineViewportState } from "./timeline-viewport";

export function TimelineViewportContent({
  children,
  viewport,
}: {
  children: ReactNode;
  viewport: TimelineViewportState;
}) {
  return (
    <div
      className="absolute inset-y-0 left-0"
      style={{
        transform: `translateX(${(-viewport.panOffset * 100).toString()}%)`,
        width: `${(viewport.zoom * 100).toString()}%`,
      }}
    >
      {children}
    </div>
  );
}
