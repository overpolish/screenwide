// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { convertFileSrc } from "@tauri-apps/api/core";
import { useCallback, useEffect, useRef, useState } from "react";

import { getProjectThumbnail } from "./api";
import { showsWords, type ProjectSummary } from "./types";

/** The stills of the projects listed, by manifest, each asked for once per
 * edit, and how to ask for one again when it is redrawn. Notes and scripts
 * have none: their cards show their words. */
export function useProjectThumbnails(projects: ProjectSummary[] | null) {
  const [thumbnails, setThumbnails] = useState<Record<string, string | null>>(
    {},
  );
  // Stills already asked for, by manifest and edit time, so a refresh asks
  // again only for a project whose movie may have changed.
  const requestedRef = useRef(new Set<string>());

  // Each fetch gets its own address: the editor redraws a still in place,
  // and the webview would otherwise keep showing the copy it has.
  const load = useCallback((file: string, key?: string) => {
    getProjectThumbnail(file)
      .then((path) => {
        setThumbnails((current) => ({
          ...current,
          [file]: path
            ? `${convertFileSrc(path)}?v=${String(Date.now())}`
            : null,
        }));
      })
      .catch(() => {
        if (key) requestedRef.current.delete(key);
      });
  }, []);

  useEffect(() => {
    for (const project of projects ?? []) {
      const key = `${project.file}@${String(project.modifiedMs)}`;
      if (showsWords(project.kind)) continue;
      if (!project.available || requestedRef.current.has(key)) continue;
      requestedRef.current.add(key);
      load(project.file, key);
    }
  }, [load, projects]);

  return { load, setThumbnails, thumbnails };
}
