// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef } from "react";

import { useColorPanel, usesSystemColorPanel } from "./use-color-panel";

/**
 * The system's colour picker, opened from a press. On the Mac app that is
 * AppKit's Colours panel; everywhere else the browser's own picker, opened
 * from a colour input kept out of sight and placed under the control that
 * asked, so the picker appears beside it. Every step of a drag is reported.
 */
export function useColorPicker(onChange: (value: string) => void) {
  const onChangeRef = useRef(onChange);
  onChangeRef.current = onChange;
  const panel = useColorPanel({
    onChange: (value) => {
      onChangeRef.current(value);
    },
  });
  const inputRef = useRef<HTMLInputElement | null>(null);

  useEffect(
    () => () => {
      inputRef.current?.remove();
    },
    [],
  );

  const openInput = (color: string, anchor: Element) => {
    let input = inputRef.current;
    if (!input) {
      input = document.createElement("input");
      input.type = "color";
      input.tabIndex = -1;
      input.setAttribute("aria-hidden", "true");
      Object.assign(input.style, {
        border: "0",
        height: "0",
        opacity: "0",
        padding: "0",
        pointerEvents: "none",
        position: "fixed",
        width: "0",
      });
      input.addEventListener("input", () => {
        onChangeRef.current(input?.value.toUpperCase() ?? color);
      });
      document.body.append(input);
      inputRef.current = input;
    }
    const { bottom, left } = anchor.getBoundingClientRect();
    input.style.left = `${String(left)}px`;
    input.style.top = `${String(bottom)}px`;
    // The input takes `#rrggbb` only, so an alpha byte is left off.
    input.value = color.slice(0, 7).toLowerCase();
    input.showPicker();
  };

  return {
    /** Opens the picker on `color`, beside `anchor`. */
    open: (color: string, anchor: Element) => {
      if (usesSystemColorPanel()) panel.open(color);
      else openInput(color, anchor);
    },
  };
}
