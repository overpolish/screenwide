// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import { convertFileSrc } from "@tauri-apps/api/core";
import { useEffect, useRef, useState } from "react";

import {
  getProjectThumbnail,
  listProjects,
} from "../../../project-browser/api";
import { RECENT, sourceFor } from "../../../project-browser/types";

import type { ProjectSummary } from "../../../project-browser/types";

const report = (what: string) => (error: unknown) => {
  console.error(what, error);
};

/** Whether a cue can be made of `project`: a recording with a picture, or
 * a screenshot. */
export const canCue = ({ available, kind }: ProjectSummary) =>
  available &&
  (kind === "screen" || kind === "camera" || kind === "screenshot");

/** The recent projects `canChoose` lets through, newest first, and their
 * stills as each arrives: by default those a cue can be made of. */
export function useCueSources(
  canChoose: (project: ProjectSummary) => boolean = canCue,
) {
  const [projects, setProjects] = useState<ProjectSummary[] | null>(null);
  const [stills, setStills] = useState<Record<string, string>>({});
  const canChooseRef = useRef(canChoose);
  canChooseRef.current = canChoose;

  useEffect(() => {
    let disposed = false;
    listProjects(sourceFor(RECENT))
      .then((listed) => {
        if (disposed) return;
        const cueable = listed
          .filter((project) => canChooseRef.current(project))
          .sort((a, b) => (b.createdMs ?? 0) - (a.createdMs ?? 0));
        setProjects(cueable);
        for (const project of cueable)
          getProjectThumbnail(project.file)
            .then((path) => {
              if (!disposed && path)
                setStills((current) => ({
                  ...current,
                  [project.file]: convertFileSrc(path),
                }));
            })
            .catch(report("Could not read a still"));
      })
      .catch(report("Could not list the projects"));
    return () => {
      disposed = true;
    };
  }, []);

  return { projects, stills };
}
