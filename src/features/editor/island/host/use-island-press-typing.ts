// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";
import { useEffect } from "react";

import { editsText } from "../../shortcuts/keyboard-target";

import type { EditorKind } from "../../types";

/** Hands the keyboard back to a box being typed into in the editor beneath
 * once a press on the island is done. On macOS the press makes the island's
 * window key while the box keeps its place as the editor's first responder,
 * so the box stays open but its keys would reach the island's shortcuts. A
 * press that leaves a field of the island being typed into keeps the
 * keyboard here. */
export function useIslandPressTyping(workspace: EditorKind) {
  useEffect(() => {
    let timer: number | undefined;
    const onPointerUp = () => {
      window.clearTimeout(timer);
      // After the press's own handlers, which may have put focus in a field.
      timer = window.setTimeout(() => {
        if (editsText(document.activeElement)) return;
        void invoke<null>("resume_annotation_typing", { workspace }).catch(
          () => undefined,
        );
      });
    };
    window.addEventListener("pointerup", onPointerUp, true);
    return () => {
      window.removeEventListener("pointerup", onPointerUp, true);
      window.clearTimeout(timer);
    };
  }, [workspace]);
}
