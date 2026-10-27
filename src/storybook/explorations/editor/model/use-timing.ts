// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useMemo, useState } from "react";

import { type Beat, type Recording } from "../recording";

import {
  type Cut,
  DEAD_AIR_S,
  type Pause,
  type PauseSetting,
  timeMap,
} from "./timeline-model";
import { breathed, trimmedTo, wordEdge } from "./trims";

// How the recording plays: what is cut, how far the squeeze has gone, and
// the pauses set by hand. Everything else the editor holds is drawn on top
// of this timing.

/** The timing as a whole, as undo takes it back. */
export type Timing = {
  cuts: Cut[];
  settings: Record<string, PauseSetting>;
  squeeze: number;
};

export function useTiming({
  beats,
  isVoiced,
  recording,
}: {
  beats: Beat[];
  isVoiced: boolean;
  recording: Recording;
}) {
  const [cuts, setCuts] = useState<Cut[]>([]);
  const [squeeze, setSqueeze] = useState(0);
  // Pauses set on their own, by id: kept as recorded, or retimed.
  const [settings, setSettings] = useState<Record<string, PauseSetting>>({});
  const map = useMemo(
    () => timeMap({ beats, cuts, isVoiced, recording, settings, squeeze }),
    [beats, cuts, isVoiced, recording, settings, squeeze],
  );
  const voice = isVoiced ? recording : null;

  const cut = (next: Cut) => {
    setCuts((current) => [...current.filter(({ id }) => id !== next.id), next]);
  };

  /** Sets a pause on its own, or with null gives it back to the squeeze.
   * The squeeze's amount moves by what this pause gave it or will give it,
   * so its level, and every other pause, stays as it was. */
  const setPause = (id: string, setting: PauseSetting | null) => {
    const pause = map.pauses.find((item) => item.id === id);
    const takes = pause ? map.squeezeTakes(pause) : 0;
    const change = (setting ? 0 : takes) - (id in settings ? 0 : takes);
    setSqueeze((current) => Math.max(0, current + change));
    setSettings((current) => {
      const { [id]: _, ...rest } = current;
      return setting ? { ...rest, [id]: setting } : rest;
    });
  };

  /** Cuts a pause back, which plays what is left as recorded. */
  const trim = (
    id: string,
    cutsFor: (pause: Pause) => (current: Cut[]) => Cut[],
  ) => {
    const pause = map.pauses.find((item) => item.id === id);
    if (!pause) return;
    setCuts(cutsFor(pause));
    setPause(id, { kind: "keep" });
  };

  return {
    cut,
    /** Cuts the stretch between two recorded moments, its edges moved off
     * any word they would split. */
    cutRange: (from: number, to: number) => {
      const start = wordEdge(Math.min(from, to), voice);
      const end = wordEdge(Math.max(from, to), voice);
      if (end - start > 0.05)
        cut({ end, id: `range-${start.toFixed(2)}`, start });
    },
    cuts,
    map,
    /** Puts back the cuts that closed up at one seam. */
    restore: (ids: string[]) => {
      setCuts((current) => current.filter(({ id }) => !ids.includes(id)));
    },
    /** Takes the timing back to how it was, for undo. */
    restoreTiming: (timing: Timing) => {
      setCuts(timing.cuts);
      setSettings(timing.settings);
      setSqueeze(timing.squeeze);
    },
    setPause,
    setSqueeze,
    settings,
    squeeze,
    /** Cuts every dead air's still time down to a breath: on a face, where
     * the squeeze can hurry silence only a little, this is what tightens a
     * recording. */
    trimDeadAir: () => {
      for (const pause of map.pauses.filter(
        ({ still }) => still > DEAD_AIR_S,
      )) {
        trim(
          pause.id,
          (found) => (current) => breathed(current, found, map.pieces),
        );
      }
    },
    /** Trims a pause's end back to a recorded moment, or drawn back out to
     * where it first ended, lets the trim go. */
    trimPause: (id: string, to: number) => {
      trim(
        id,
        (pause) => (current) => trimmedTo(current, pause, { to, voice }),
      );
    },
    /** Cuts a pause's still time down to a breath. */
    trimToBreath: (id: string) => {
      trim(id, (pause) => (current) => breathed(current, pause, map.pieces));
    },
  };
}
