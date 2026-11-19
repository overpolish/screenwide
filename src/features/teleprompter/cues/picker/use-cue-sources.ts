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
const canCue = ({ available, kind }: ProjectSummary) =>
  available &&
  (kind === "screen" || kind === "camera" || kind === "screenshot");

/** The recent projects `canChoose` lets through, newest first: by default
 * those a cue can be made of. Null until listed. */
export function useCueSources(
  canChoose: (project: ProjectSummary) => boolean = canCue,
) {
  const [projects, setProjects] = useState<ProjectSummary[] | null>(null);
  const canChooseRef = useRef(canChoose);
  canChooseRef.current = canChoose;

  useEffect(() => {
    let disposed = false;
    listProjects(sourceFor(RECENT))
      .then((listed) => {
        if (disposed) return;
        setProjects(
          listed
            .filter((project) => canChooseRef.current(project))
            .sort((a, b) => (b.createdMs ?? 0) - (a.createdMs ?? 0)),
        );
      })
      .catch(report("Could not list the projects"));
    return () => {
      disposed = true;
    };
  }, []);

  return projects;
}

/** The stills of `projects`, by manifest, as each arrives. */
export function useCueStills(projects: readonly ProjectSummary[] | null) {
  const [stills, setStills] = useState<Record<string, string>>({});

  useEffect(() => {
    let disposed = false;
    for (const project of projects ?? [])
      getProjectThumbnail(project.file)
        .then((path) => {
          if (!disposed && path)
            setStills((current) => ({
              ...current,
              [project.file]: convertFileSrc(path),
            }));
        })
        .catch(report("Could not read a still"));
    return () => {
      disposed = true;
    };
  }, [projects]);

  return stills;
}
