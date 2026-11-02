// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type ReactNode, type Ref } from "react";

import { ScrollArea } from "../../../primitives/scroll-area/scroll-area";

// A pane: its picture held at the top, if it has one, and its rows
// scrolling under it, so the picture stays in view while any row is read
// or changed. The column runs to the window's end and bottom edges so the
// scroll fade lands there, and carries the window's inset on what it
// holds instead.

export function SettingsPane({
  children,
  label,
  paneRef,
  stage,
}: {
  children: ReactNode;
  label: string;
  /** The rows' section, where a search result looks for its row. */
  paneRef?: Ref<HTMLElement>;
  stage?: ReactNode;
}) {
  return (
    <div className="flex min-h-0 min-w-0 grow flex-col gap-separate">
      {stage ? <div className="pe-inset">{stage}</div> : null}
      <ScrollArea inWindowCorner rootClassName="min-h-0 min-w-0 grow">
        <section
          aria-label={label}
          className="flex flex-col gap-group pe-inset pb-inset"
          ref={paneRef}
        >
          {children}
        </section>
      </ScrollArea>
    </div>
  );
}
