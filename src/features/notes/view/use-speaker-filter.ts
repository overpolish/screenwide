// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";

// Which speakers' words alone the page shows, and which stretches of
// everyone else's were opened again. Choosing or letting go of a speaker
// closes every stretch afresh. A speaker no longer in the notes is no longer
// chosen; one renamed stays chosen under their new name.

export function useSpeakerFilter(speakers: readonly string[]) {
  const [state, setState] = useState<{
    chosen: readonly string[];
    /** The first turn of each hidden stretch opened again. */
    opened: readonly number[];
  }>({ chosen: [], opened: [] });
  return {
    chosen: state.chosen.filter((name) => speakers.includes(name)),
    open: (start: number) => {
      setState(({ chosen, opened }) => ({
        chosen,
        opened: [...opened, start],
      }));
    },
    opened: state.opened,
    rename: (from: string, to: string) => {
      setState(({ chosen, opened }) => ({
        chosen: [...new Set(chosen.map((name) => (name === from ? to : name)))],
        opened,
      }));
    },
    toggle: (name: string) => {
      setState(({ chosen }) => ({
        chosen: chosen.includes(name)
          ? chosen.filter((chose) => chose !== name)
          : [...chosen, name],
        opened: [],
      }));
    },
  };
}
