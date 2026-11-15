// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect } from "react";

import { useSettingsApi } from "./settings-api-context";

/** Switches Settings to the pane another window asks for, such as Models
 * when dictation cannot start. */
export function useSectionOpen(onOpen: (section: string) => void) {
  const { listenToSectionOpen } = useSettingsApi();
  useEffect(() => {
    let stopped = false;
    let stop: (() => void) | undefined;
    listenToSectionOpen(onOpen)
      .then((unlisten) => {
        if (stopped) unlisten();
        else stop = unlisten;
      })
      .catch(() => {
        // Without the event, the pane is still one click away in the sidebar.
      });
    return () => {
      stopped = true;
      stop?.();
    };
  }, [listenToSectionOpen, onOpen]);
}
