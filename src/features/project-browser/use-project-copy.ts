// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";

import { COPY_PROGRESS_EVENT } from "./api";

import type { CopyingProjects } from "./project-browser-notices";
import type { CopyProgress } from "../../bindings/CopyProgress";

/**
 * The move or duplicate under way, as the notice shows it: what it is, and
 * how far the copy has got by the progress the app reports. One at a time;
 * the browser offers neither while one runs.
 */
export function useProjectCopy(run: (action: Promise<unknown>) => void) {
  const [copying, setCopying] = useState<CopyingProjects | null>(null);

  useEffect(() => {
    const listening = listen<CopyProgress>(
      COPY_PROGRESS_EVENT,
      ({ payload: { copiedBytes, totalBytes } }) => {
        setCopying(
          (current) =>
            current && {
              ...current,
              fraction: totalBytes > 0 ? copiedBytes / totalBytes : 1,
            },
        );
      },
    );
    return () => {
      void listening.then((unlisten) => {
        unlisten();
      });
    };
  }, []);

  return {
    copying,
    start: (
      copy: Omit<CopyingProjects, "fraction">,
      action: Promise<unknown>,
    ) => {
      setCopying({ ...copy, fraction: 0 });
      run(
        action.finally(() => {
          setCopying(null);
        }),
      );
    },
  };
}
