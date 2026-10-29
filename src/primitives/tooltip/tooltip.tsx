// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Ref } from "react";

import { Keyboard, Shortcut } from "../keyboard/keyboard";

type TooltipProps = {
  label: string;
  /** The shortcut's keys, as `hotkeyKeys` returns them. */
  keys?: string[];
  ref?: Ref<HTMLDivElement>;
};

// A tooltip's content, the whole of the tooltip window, drawn in the
// island's black and dark dress as a menu opened from an island is: the
// window carries no material or shadow (`tooltip_window/creation.rs`), and
// the faint inset ring keeps its edge over the island's own black. The label
// is in a button's label type, with the shortcut that does the same thing as
// keycaps after it. It shrink-wraps the words up to the window's widest
// (360px, `tooltip_window.rs`), so the window can be measured and sized to
// it; long text, such as a path, wraps anywhere rather than running past it.
export function Tooltip({ keys, label, ref }: TooltipProps) {
  return (
    <div
      className="dark inline-flex w-max max-w-90 items-center gap-inline rounded-control bg-island px-inset-sm py-1 text-label text-fg inset-ring inset-ring-fill-strong"
      ref={ref}
    >
      <span className="min-w-0 wrap-anywhere">{label}</span>
      {keys?.length ? (
        <Shortcut>
          {keys.map((key, index) => (
            <Keyboard key={`${key}-${index.toString()}`}>{key}</Keyboard>
          ))}
        </Shortcut>
      ) : null}
    </div>
  );
}
