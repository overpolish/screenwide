// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import { useEffect } from "react";

import { useSettingsApi } from "../settings-api-context";

/** Switches Settings to the Licences pane whenever a paid tool is turned
 * away, so its key can be added or bought. */
export function useLicencesOpen(onOpen: () => void) {
  const { listenToLicencesOpen } = useSettingsApi();
  useEffect(() => {
    let stopped = false;
    let stop: (() => void) | undefined;
    listenToLicencesOpen(onOpen)
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
  }, [listenToLicencesOpen, onOpen]);
}
