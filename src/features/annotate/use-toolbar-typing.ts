// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { isTauri } from "@tauri-apps/api/core";
import { useEffect } from "react";

import { beginAnnotateToolbarTyping, endAnnotateToolbarTyping } from "./api";

const isField = (node: EventTarget | null) =>
  node instanceof HTMLElement &&
  (node instanceof HTMLInputElement ||
    node instanceof HTMLTextAreaElement ||
    node.isContentEditable);

/**
 * Asks for the keyboard while a field on the plate has focus, and gives it back
 * when none has.
 *
 * The toolbar is a window that refuses key status until a view on it needs the
 * keyboard, which is what leaves the letters that pick up a tool working while
 * its buttons are pressed. A field is the one view that does need it, and only
 * the page knows when one is focused.
 *
 * Moving between two fields is one handover, not a release and a claim: the
 * blur names where focus is going, so a move within the plate is left alone.
 */
export function useToolbarTyping() {
  useEffect(() => {
    if (!isTauri()) return;
    let typing = false;

    const claim = () => {
      if (typing) return;
      typing = true;
      beginAnnotateToolbarTyping().catch(() => {
        typing = false;
      });
    };
    const release = () => {
      if (!typing) return;
      typing = false;
      endAnnotateToolbarTyping().catch(() => undefined);
    };

    const onFocusIn = (event: FocusEvent) => {
      if (isField(event.target)) claim();
    };
    const onFocusOut = (event: FocusEvent) => {
      if (!isField(event.target) || isField(event.relatedTarget)) return;
      release();
    };

    document.addEventListener("focusin", onFocusIn);
    document.addEventListener("focusout", onFocusOut);
    return () => {
      document.removeEventListener("focusin", onFocusIn);
      document.removeEventListener("focusout", onFocusOut);
      release();
    };
  }, []);
}
