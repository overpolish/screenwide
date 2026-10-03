// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";

import {
  chosenAnnotationIds,
  toggledAnnotationIds,
} from "../../annotations/annotation-selection";
import { sweptTimelineItems } from "../../timeline/tracks/timeline-item-selection";

import { RecordingAnnotationClip } from "./recording-annotations";

/**
 * The annotation lane's choice kept by the rules the editor keeps it by, for
 * the stories and previews that have no editor behind the lane. The names
 * match the lane's own props.
 */
export function useLaneAnnotationSelection(
  clips: RecordingAnnotationClip[],
  initial: readonly string[] = [],
) {
  const [chosen, setChosen] = useState<ReadonlySet<string>>(
    () => new Set(initial),
  );
  const groupable = (id: string) =>
    !clips.some((clip) => clip.annotation.id === id && clip.pin);
  const selectedIds = chosenAnnotationIds(
    clips.map((clip) => clip.annotation.id),
    chosen,
    groupable,
  );
  return {
    onClearSelection: () => {
      setChosen(new Set());
    },
    onSelect: (id: string, toggle: boolean) => {
      setChosen(toggledAnnotationIds(selectedIds, id, { groupable, toggle }));
    },
    onSelectSwept: (ids: string[], additive: boolean) => {
      setChosen(sweptTimelineItems(selectedIds, ids, additive));
    },
    selectedIds,
  };
}
