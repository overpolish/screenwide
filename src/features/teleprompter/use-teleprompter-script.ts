// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";

import {
  readScript,
  Script,
  TELEPROMPTER_MOVED_EVENT,
  TELEPROMPTER_SCRIPT_EVENT,
  teleprompterScript,
} from "./api";

export type ShownScript = {
  /** Counts the scripts opened, so each is read anew from its start. */
  opened: number;
  /** The script's project, by its manifest, which a rename moves. */
  project: string;
  /** The script as it was read when it opened. */
  script: Script;
};

/**
 * The script the teleprompter shows: the one it was opened on, then any it
 * is asked to show instead, each read once as it opens; after that the
 * teleprompter is where it is written. A rename moves its project, which
 * is followed without reading it again.
 */
export function useTeleprompterScript() {
  const [shown, setShown] = useState<ShownScript | null>(null);

  useEffect(() => {
    let disposed = false;
    let asked: string | null = null;
    const open = (project: string) => {
      asked = project;
      readScript(project)
        .then((script) => {
          if (disposed || asked !== project) return;
          setShown((current) => ({
            opened: (current?.opened ?? 0) + 1,
            project,
            script,
          }));
        })
        .catch((error: unknown) => {
          console.error("Could not read the script", error);
        });
    };
    const unlisteners = [
      listen<string>(TELEPROMPTER_SCRIPT_EVENT, ({ payload }) => {
        open(payload);
      }),
      listen<string>(TELEPROMPTER_MOVED_EVENT, ({ payload }) => {
        asked = payload;
        setShown((current) => current && { ...current, project: payload });
      }),
    ];
    void teleprompterScript().then((first) => {
      if (first !== null && !disposed && asked === null) open(first);
    });
    return () => {
      disposed = true;
      for (const unlistening of unlisteners)
        void unlistening.then((unlisten) => {
          unlisten();
        });
    };
  }, []);

  return shown;
}
