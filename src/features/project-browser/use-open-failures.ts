// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect } from "react";

import { t } from "../../i18n/i18n";
import { useTauriEvent } from "../../lib/use-tauri-event";

import { PROJECT_OPEN_FAILED_EVENT, takeProjectOpenFailures } from "./api";

/** Why a project would not open, as the browser's island says it. */
export const openFailure = (reason: string) =>
  t("project-browser-open-failed", { reason });

/** `open`, failing with the browser's own sentence when the project would
 * not open. */
export const opening = (open: Promise<unknown>) =>
  open.catch((cause: unknown) => {
    throw new Error(
      openFailure(cause instanceof Error ? cause.message : String(cause)),
    );
  });

/**
 * Says on the island why a project asked for from outside the app would not
 * open: by Finder, Explorer or a second launch, which bring the browser out
 * for it. Rust keeps the reasons until they are taken, as the page may not
 * be listening yet at launch; several are said one to a line.
 */
export function useOpenFailures(show: (error: string) => void) {
  useEffect(() => {
    take(show);
  }, [show]);
  useTauriEvent(PROJECT_OPEN_FAILED_EVENT, () => {
    take(show);
  });
}

const take = (show: (error: string) => void) => {
  takeProjectOpenFailures()
    .then((reasons) => {
      if (reasons.length > 0) show(reasons.map(openFailure).join("\n"));
    })
    .catch((cause: unknown) => {
      console.error("Could not read why a project would not open", cause);
    });
};
