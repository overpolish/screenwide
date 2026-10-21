// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type ReactNode, useId } from "react";

// A setting's row: its name, an optional line under it, and the control at
// the end. The control takes the row's name through `aria-labelledby`. A row
// whose value differs from the default carries a dot after its name; pressing
// the dot resets the row. A sketch for ControlRow's redesign, kept to the
// explorations until then.
export function SettingRow({
  children,
  description,
  isChanged = false,
  title,
}: {
  children: (labelledBy: string) => ReactNode;
  title: string;
  description?: string;
  isChanged?: boolean;
}) {
  const id = useId();
  return (
    <div className="flex min-h-control-height items-center gap-separate">
      <div className="flex min-w-0 grow flex-col">
        <span className="flex items-center gap-inline">
          <span className="text-body text-fg" id={id}>
            {title}
          </span>
          {isChanged ? (
            <span
              aria-label={`Reset ${title}`}
              className="size-1.5 rounded-full bg-primary"
              role="button"
            />
          ) : null}
        </span>
        {description ? (
          <span className="text-caption text-fg-tertiary">{description}</span>
        ) : null}
      </div>
      <div className="flex shrink-0 items-center gap-inline">
        {children(id)}
      </div>
    </div>
  );
}
