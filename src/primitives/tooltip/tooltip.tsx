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

// A tooltip's content, the whole of the tooltip window: the window supplies
// the material, corner and shadow, so this paints no surface. The label is in
// a button's label type, with the shortcut that does the same thing as
// keycaps after it. It shrink-wraps the words up to the window's widest
// (360px, `tooltip_window.rs`), so the window can be measured and sized to it;
// long text, such as a path, wraps anywhere rather than running past it.
export function Tooltip({ keys, label, ref }: TooltipProps) {
  return (
    <div
      className="inline-flex w-max max-w-90 items-center gap-inline px-inset-sm py-1 text-label text-fg"
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
