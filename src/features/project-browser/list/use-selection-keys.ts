// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef } from "react";

import { isWindows } from "../../../lib/platform";
import { ownsTextEditingKeys } from "../../editor/shortcuts/keyboard-target";

import type { ProjectActions } from "./use-project-menu";

/** The keys that act on the choice of projects, anywhere in the window but
 * a text field: Command-A (Control-A on Windows) chooses all, Escape none,
 * and Command-Backspace (Delete on Windows) deletes what is chosen. */
export function useSelectionKeys({
  actions,
  onClear,
  onSelectAll,
  selectedFiles,
}: {
  actions: ProjectActions;
  onClear: () => void;
  onSelectAll: () => void;
  selectedFiles: string[];
}) {
  const keysRef = useRef<(event: KeyboardEvent) => void>(() => undefined);
  useEffect(() => {
    keysRef.current = (event) => {
      if (ownsTextEditingKeys(event.target)) return;
      const command = isWindows() ? event.ctrlKey : event.metaKey;
      if (command && event.key.toLowerCase() === "a") {
        event.preventDefault();
        onSelectAll();
      } else if (event.key === "Escape" && selectedFiles.length > 0) {
        onClear();
      } else if (
        selectedFiles.length > 0 &&
        "onDelete" in actions &&
        (isWindows()
          ? event.key === "Delete"
          : event.metaKey && event.key === "Backspace")
      ) {
        event.preventDefault();
        actions.onDelete(selectedFiles);
      }
    };
  });
  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      keysRef.current(event);
    };
    window.addEventListener("keydown", onKeyDown);
    return () => {
      window.removeEventListener("keydown", onKeyDown);
    };
  }, []);
}
