// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useCallback, useRef, useState } from "react";

import { Axis } from "./gradient-field";
import { DistanceProbe, Point } from "./pixel-analysis";

export function useProbeDrag({
  onFinish,
  preview,
}: {
  onFinish: (probe: DistanceProbe) => void;
  preview: (axis: Axis, start: Point, end: Point) => DistanceProbe | undefined;
}) {
  const [draft, setDraft] = useState<DistanceProbe>();
  const draftRef = useRef<DistanceProbe | undefined>(undefined);
  const gestureRef = useRef<{ axis: Axis; start: Point } | null>(null);

  const updateDraft = useCallback((next: DistanceProbe | undefined) => {
    draftRef.current = next;
    setDraft(next);
  }, []);
  const begin = useCallback(
    (axis: Axis, world: Point) => {
      gestureRef.current = { axis, start: world };
      updateDraft(preview(axis, world, world));
    },
    [preview, updateDraft],
  );
  const move = useCallback(
    (world: Point) => {
      const gesture = gestureRef.current;
      if (!gesture) return;
      updateDraft(preview(gesture.axis, gesture.start, world));
    },
    [preview, updateDraft],
  );
  const cancel = useCallback(() => {
    gestureRef.current = null;
    updateDraft(undefined);
  }, [updateDraft]);
  const finish = useCallback(() => {
    const committed = draftRef.current;
    cancel();
    if (committed) onFinish(committed);
  }, [cancel, onFinish]);

  return {
    begin,
    cancel,
    draft,
    finish,
    isActive: () => gestureRef.current !== null,
    move,
  };
}
