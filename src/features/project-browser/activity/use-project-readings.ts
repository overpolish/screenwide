// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useMemo, useRef, useState } from "react";

import { getProjectActivity } from "../api";

import { projectReading, type ProjectReading } from "./project-reading";

import type { ProjectSummary } from "../types";

/** How many projects are read at once: each reads its cursor and key files
 * and its transcript, so a long list is read a few at a time rather than all
 * at once. */
const AT_ONCE = 3;

const keyOf = ({ file, modifiedMs }: ProjectSummary) =>
  `${file}@${String(modifiedMs)}`;

/**
 * Each listed recording's reading, by manifest: null for one that has none
 * (a screenshot, or a project that cannot be read), absent while it is read.
 * A project is read again once it has been edited since, as its edit or
 * transcript may have changed; one read before, in another source, is not
 * read twice.
 */
export function useProjectReadings(projects: readonly ProjectSummary[] | null) {
  const [read, setRead] = useState<ReadonlyMap<string, ProjectReading | null>>(
    () => new Map(),
  );
  const askedRef = useRef(new Set<string>());
  // Readings land one by one; each would draw every card again, so those
  // that land within a frame are taken together.
  const landedRef = useRef(new Map<string, ProjectReading | null>());
  const frameRef = useRef<number | null>(null);
  useEffect(
    () => () => {
      if (frameRef.current !== null) cancelAnimationFrame(frameRef.current);
    },
    [],
  );
  useEffect(() => {
    const queue = (projects ?? []).filter(
      (project) =>
        project.available &&
        project.kind !== "screenshot" &&
        !askedRef.current.has(keyOf(project)),
    );
    for (const project of queue) askedRef.current.add(keyOf(project));
    const land = (key: string, reading: ProjectReading | null) => {
      landedRef.current.set(key, reading);
      frameRef.current ??= requestAnimationFrame(() => {
        frameRef.current = null;
        const landed = landedRef.current;
        landedRef.current = new Map();
        setRead((current) => new Map([...current, ...landed]));
      });
    };
    const next = async (): Promise<void> => {
      const project = queue.shift();
      if (!project) return;
      const key = keyOf(project);
      try {
        const activity = await getProjectActivity(project.file);
        const reading = activity ? projectReading(activity) : null;
        land(key, reading);
      } catch (cause: unknown) {
        // Asked for again the next time the list is read.
        askedRef.current.delete(key);
        console.error("Could not read what the project holds", cause);
      }
      return next();
    };
    for (let lane = 0; lane < AT_ONCE; lane += 1) void next();
  }, [projects]);

  return useMemo(
    () =>
      Object.fromEntries(
        (projects ?? []).flatMap((project) => {
          const reading = read.get(keyOf(project));
          return reading === undefined ? [] : [[project.file, reading]];
        }),
      ) as Record<string, ProjectReading | null>,
    [projects, read],
  );
}
