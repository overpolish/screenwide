// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useCallback, useMemo, useRef, useState } from "react";

export type TimelineItemSelection<ItemId> = {
  ids: ReadonlySet<ItemId>;
  onClear: () => void;
  onSelect: (itemId: ItemId, toggle: boolean) => void;
};

export function selectTimelineItem<ItemId>(
  current: ReadonlySet<ItemId>,
  itemId: ItemId,
  toggle: boolean,
) {
  if (!toggle) return new Set([itemId]);
  const next = new Set(current);
  if (next.has(itemId)) next.delete(itemId);
  else next.add(itemId);
  return next;
}

export function useTimelineItemSelection<ItemId>(
  onSelectionStart?: () => void,
): TimelineItemSelection<ItemId> {
  const [ids, setIds] = useState(() => new Set<ItemId>());
  const onSelectionStartRef = useRef(onSelectionStart);
  onSelectionStartRef.current = onSelectionStart;
  const onClear = useCallback(() => {
    setIds(new Set());
  }, []);
  const onSelect = useCallback((itemId: ItemId, toggle: boolean) => {
    onSelectionStartRef.current?.();
    setIds((current) => selectTimelineItem(current, itemId, toggle));
  }, []);
  return useMemo(() => ({ ids, onClear, onSelect }), [ids, onClear, onSelect]);
}
