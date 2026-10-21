// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ReactNode } from "react";

// A titled group of a settings pane's rows. Groups are told apart by spacing
// alone: no box, fill or rule around them.
export function SettingsGroup({
  children,
  title,
}: {
  children: ReactNode;
  title: ReactNode;
}) {
  return (
    <section className="flex flex-col gap-inline">
      <h2 className="text-heading text-fg-secondary">{title}</h2>
      <div className="flex flex-col gap-separate">{children}</div>
    </section>
  );
}
