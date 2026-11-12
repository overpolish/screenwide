// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { isTauri } from "@tauri-apps/api/core";
import { emit } from "@tauri-apps/api/event";

import { isMacHost } from "../../../lib/platform";

// A Mac's webview hides the pointer until it moves whenever the page takes a
// key (WebKit's `doneWithKeyEvent`), as a text field does while typing. The
// editor's keys are about what the pointer does next, a tool taken up or a
// mode opened over the picture, so the native side is told to show it again
// (`src-tauri/src/editor/pointer.rs`). It is told on a later task: the
// webview hides the pointer when it hears the key was taken, which it hears
// as the key's dispatch ends.

let isAsked = false;

/** Brings the pointer back after a key the editor took. */
export const keepPointerShown = () => {
  if (!isMacHost || !isTauri() || isAsked) return;
  isAsked = true;
  setTimeout(() => {
    isAsked = false;
    // Only how it looks: a pointer left hidden comes back as it moves.
    emit("editor://key-taken").catch(() => undefined);
  }, 0);
};
