// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { cancelExportJob } from "../api";

import { failExport } from "./export-island-store";

/** Asks the running export to stop. `setCanceling` hears that it is
 * stopping, and that it no longer is where the export refused or the ask
 * failed, which then shows as the export's failure. */
export const cancelActiveExport = (
  setCanceling: (canceling: boolean) => void,
) => {
  setCanceling(true);
  cancelExportJob()
    .then((accepted) => {
      if (!accepted) setCanceling(false);
    })
    .catch((cause: unknown) => {
      console.error("Could not cancel the active export", cause);
      failExport(cause instanceof Error ? cause.message : String(cause));
      setCanceling(false);
    });
};
