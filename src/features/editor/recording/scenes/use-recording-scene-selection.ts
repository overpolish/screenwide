// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useCallback, useState } from "react";

import { useEditorWindowShortcuts } from "../../shortcuts/use-editor-window-shortcuts";
import {
  selectTimelineItem,
  sweptTimelineItems,
} from "../../timeline/tracks/timeline-item-selection";
import { useEditorEditGesture } from "../../use-editor-edit-history";

import { RecordingSceneClip } from "./recording-scenes";

const NOTHING_CHOSEN: ReadonlySet<string> = new Set();

export type RecordingSceneSelection = {
  ids: ReadonlySet<string>;
  onClear: () => void;
  /** Choose the scene `id` alone, or with `toggle`, add or take it away. */
  onSelect: (id: string, toggle: boolean) => void;
  /** Choose the scenes a band swept over, alone or added to the choice. */
  onSelectSwept: (ids: string[], additive: boolean) => void;
};

/**
 * The scenes chosen in their lane, which Delete takes away together as one
 * step and Escape lets go. A chosen scene that leaves the recording, deleted
 * or undone away, leaves the choice with it, so undoing it back never brings
 * it back chosen.
 *
 * Once scenes are chosen, the choice follows the playhead the way the Scene
 * panel does: the scene it moves onto is chosen alone, and none while it is
 * between scenes. It follows until the choice is let go, deleted or taken
 * by a choice elsewhere; until the playhead moves onto another scene, a
 * choice of several stays as it was made.
 */
export function useRecordingSceneSelection({
  clips,
  commit,
  currentId,
}: {
  clips: readonly RecordingSceneClip[];
  commit: (clips: RecordingSceneClip[]) => void;
  /** The scene under the playhead, null between scenes. */
  currentId: string | null;
}): RecordingSceneSelection {
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
