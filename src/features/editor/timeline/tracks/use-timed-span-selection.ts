// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useCallback, useState } from "react";

import { useEditorWindowShortcuts } from "../../shortcuts/use-editor-window-shortcuts";
import { useEditorEditGesture } from "../../use-editor-edit-history";

import { TimedSpanClip } from "./timed-span-clips";
import {
  selectTimelineItem,
  sweptTimelineItems,
} from "./timeline-item-selection";

const NOTHING_CHOSEN: ReadonlySet<string> = new Set();

export type TimedSpanSelection = {
  ids: ReadonlySet<string>;
  onClear: () => void;
  /** Choose the clip `id` alone, or with `toggle`, add or take it away. */
  onSelect: (id: string, toggle: boolean) => void;
  /** Choose the clips a band swept over, alone or added to the choice. */
  onSelectSwept: (ids: string[], additive: boolean) => void;
};

/** `selection`'s choosing, each choice made after `letGo` has let go of
 * what else was chosen, so Delete only ever takes away one kind of thing. A
 * band that sweeps up nothing lets nothing go. */
export const lettingGoFirst = (
  selection: Pick<TimedSpanSelection, "onSelect" | "onSelectSwept">,
  letGo: () => void,
): Pick<TimedSpanSelection, "onSelect" | "onSelectSwept"> => ({
  onSelect: (id, toggle) => {
    letGo();
    selection.onSelect(id, toggle);
  },
  onSelectSwept: (ids, additive) => {
    if (ids.length > 0) letGo();
    selection.onSelectSwept(ids, additive);
  },
});

/**
 * The span clips chosen in their lane, which Delete takes away together as
 * one step and Escape lets go. A chosen clip that leaves the recording,
 * deleted or undone away, leaves the choice with it, so undoing it back never
 * brings it back chosen.
 *
 * Once clips are chosen, the choice follows the playhead the way the panel
 * that edits them does: the clip it moves onto is chosen alone, and none
 * while it is between clips. It follows until the choice is let go, deleted
 * or taken by a choice elsewhere; until the playhead moves onto another
 * clip, a choice of several stays as it was made.
 */
export function useTimedSpanSelection<Clip extends TimedSpanClip>({
  clips,
  commit,
  currentId,
}: {
  clips: readonly Clip[];
  commit: (clips: Clip[]) => void;
  /** The clip under the playhead, null between clips. */
  currentId: string | null;
}): TimedSpanSelection {
  const [ids, setIds] = useState(NOTHING_CHOSEN);
  const [isFollowing, setIsFollowing] = useState(false);
  const [followedId, setFollowedId] = useState(currentId);
  const editGesture = useEditorEditGesture();
  if (followedId !== currentId) {
    setFollowedId(currentId);
    if (isFollowing)
      setIds(currentId === null ? NOTHING_CHOSEN : new Set([currentId]));
  } else if ([...ids].some((id) => !clips.some((clip) => clip.id === id)))
    setIds(new Set([...ids].filter((id) => clips.some((c) => c.id === id))));
  const onClear = useCallback(() => {
    setIds(NOTHING_CHOSEN);
    setIsFollowing(false);
  }, []);
  const deleteSelected = useCallback(() => {
    editGesture.beginGesture();
    commit(clips.filter((clip) => !ids.has(clip.id)));
    setIds(NOTHING_CHOSEN);
    setIsFollowing(false);
    editGesture.endGesture();
  }, [clips, commit, editGesture, ids]);
  useEditorWindowShortcuts({
    onDelete: ids.size > 0 ? deleteSelected : undefined,
    onDeselect: ids.size > 0 ? onClear : undefined,
  });
  const choose = (next: ReadonlySet<string>) => {
    setIds(next);
    setIsFollowing(next.size > 0);
  };
  return {
    ids,
    onClear,
    onSelect: (id, toggle) => {
      choose(selectTimelineItem(ids, id, toggle));
    },
    onSelectSwept: (swept, additive) => {
      choose(sweptTimelineItems(ids, swept, additive));
    },
  };
}
