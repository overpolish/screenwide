// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import { convertFileSrc } from "@tauri-apps/api/core";
import { useEffect, useMemo, useRef, useState } from "react";

import { recordedTime } from "../../../editor/timeline/timeline-scale";
import {
  projectReading,
  type ProjectReading,
} from "../../../project-browser/activity/project-reading";
import {
  getProjectActivity,
  getScrubStrip,
} from "../../../project-browser/api";
import { cuePreview } from "../cue-api";

import type {
  ProjectSummary,
  ScrubStrip,
} from "../../../project-browser/types";

const report = (what: string) => (error: unknown) => {
  console.error(what, error);
};

/** A moment drawn as the edit shows it: where, in played seconds, and its
 * picture. */
export type DrawnMoment = { at: number; picture: ImageBitmap };

/**
 * A recording's moments as the cue picker scrubs them: its beats and how it
 * plays now, its scrub strip, the moment scrubbed to in played seconds, and
 * the latest moment drawn as its edit shows it.
 */
export function useCueMoment(project: ProjectSummary) {
  const [reading, setReading] = useState<ProjectReading | null>(null);
  const [strip, setStrip] = useState<ScrubStrip | null>(null);
  const [played, setPlayed] = useState(0);
  const [drawn, setDrawn] = useState<DrawnMoment | null>(null);
  const map = useMemo(
    () => reading?.map ?? recordedTime((project.durationMs ?? 0) / 1000),
    [project.durationMs, reading],
  );

  useEffect(() => {
    let disposed = false;
    getProjectActivity(project.file)
      .then((activity) => {
        if (!disposed && activity) setReading(projectReading(activity));
      })
      .catch(report("Could not read the recording's beats"));
    getScrubStrip(project.file)
      .then((file) => {
        if (!disposed && file)
          setStrip({ ...file, src: convertFileSrc(file.path) });
      })
      .catch(report("Could not read the scrub strip"));
    return () => {
      disposed = true;
    };
  }, [project.file]);

  // One drawing at a time, of the newest moment asked for, so scrubbing
  // shows the real frame as fast as it can be drawn without a queue of
  // stale ones behind it.
  const wantedRef = useRef<number | null>(null);
  const drawingRef = useRef(false);
  const mapRef = useRef(map);
  mapRef.current = map;
  const drawRef = useRef<() => void>(() => undefined);
  useEffect(() => {
    let isMounted = true;
    const draw = () => {
      const at = wantedRef.current;
      if (!isMounted || at === null || drawingRef.current) return;
      wantedRef.current = null;
      drawingRef.current = true;
      cuePreview(project.file, mapRef.current.toRecorded(at) * 1000)
        .then((picture) => {
          if (isMounted) setDrawn({ at, picture });
        })
        .catch(report("Could not draw the moment"))
        .finally(() => {
          drawingRef.current = false;
          // The newest moment asked for, by whichever drawing loop is
          // current.
          drawRef.current();
        });
    };
    drawRef.current = draw;
    draw();
    return () => {
      isMounted = false;
    };
  }, [project.file]);
  useEffect(() => {
    wantedRef.current = played;
    drawRef.current();
  }, [map, played]);

  return { drawn, map, played, reading, setPlayed, strip };
}
