// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useRef } from "react";

/**
 * The annotations in hand when the select tool is put down, chosen again
 * when it is taken back up, so turning it off and on again does not hand the
 * choice to the layer underneath. Nothing is put back over a choice made in
 * the meantime.
 *
 * Answers the call each tool change makes, with the tool it leaves and the
 * one it takes up.
 */
export function useSelectToolMemory(
  selectedIds: ReadonlySet<string>,
  choose: (ids: readonly string[]) => void,
) {
  const rememberedRef = useRef<readonly string[]>([]);
  const selectedRef = useRef(selectedIds);
  selectedRef.current = selectedIds;
  return (from: string | null, to: string | null) => {
    if (from === "select" && to !== "select") {
      rememberedRef.current = [...selectedRef.current];
      return;
    }
    if (from === "select" || to !== "select") return;
    const ids = rememberedRef.current;
    rememberedRef.current = [];
    if (ids.length > 0 && selectedRef.current.size === 0) choose(ids);
  };
}
