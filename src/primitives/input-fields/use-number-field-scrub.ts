// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Channel, invoke, isTauri } from "@tauri-apps/api/core";
import { useCallback, useEffect, useRef } from "react";

type CursorScrubEvent =
  | {
      altKey: boolean;
      deltaX: number;
      deltaY: number;
      shiftKey: boolean;
      type: "move";
    }
  | { type: "end" };

interface NumberFieldScrubOptions {
  changeValue: (value: number) => void;
  groupRef: React.RefObject<HTMLDivElement | null>;
  valueRef: React.RefObject<number>;
  step?: number;
}

export const useNumberFieldScrub = ({
  changeValue,
  groupRef,
  step,
  valueRef,
}: NumberFieldScrubOptions) => {
  const cursorScrubChannelRef = useRef<Channel<CursorScrubEvent> | null>(null);
  const cursorScrubRef = useRef<Promise<unknown> | null>(null);
  const dragRef = useRef<{
    residual: number;
    scrubbing: boolean;
    startTravel: number;
  } | null>(null);

  const moveScrub = useCallback(
    (travel: number, shiftKey: boolean, altKey: boolean) => {
      const drag = dragRef.current;
      if (!drag) return;

      if (!drag.scrubbing) {
        drag.startTravel += travel;
        if (Math.abs(drag.startTravel) < 3) return;
        drag.scrubbing = true;
      }

      drag.residual += travel;
      const stepsMoved = Math.trunc(drag.residual / 4);
      if (stepsMoved === 0) return;

      drag.residual -= stepsMoved * 4;
      const multiplier = (shiftKey ? 10 : 1) * (altKey ? 0.1 : 1);
      changeValue(valueRef.current + stepsMoved * (step ?? 1) * multiplier);
    },
    [changeValue, step, valueRef],
  );

  const releaseCursorScrub = useCallback(() => {
    document.documentElement.removeAttribute("data-number-field-scrubbing");
    cursorScrubChannelRef.current = null;
    const cursorScrub = cursorScrubRef.current;
    cursorScrubRef.current = null;
    if (cursorScrub) {
      void cursorScrub
        .catch(() => undefined)
        .then(() => invoke("end_cursor_scrub", { cursorOffsetX: null }))
        .catch(() => undefined);
    }
  }, []);

  const finishScrub = useCallback(() => {
    const drag = dragRef.current;
    if (!drag) return;

    dragRef.current = null;
    releaseCursorScrub();

    const input = groupRef.current?.querySelector("input");
    if (!drag.scrubbing) {
      input?.focus();
      input?.select();
    }
  }, [groupRef, releaseCursorScrub]);

  useEffect(() => releaseCursorScrub, [releaseCursorScrub]);

  return (event: React.PointerEvent<HTMLDivElement>) => {
    if (
      !isTauri() ||
      event.button !== 0 ||
      (event.target as Element).closest("button")
    )
      return;

    dragRef.current = {
      residual: 0,
      scrubbing: false,
      startTravel: 0,
    };
    event.preventDefault();
    document.documentElement.setAttribute("data-number-field-scrubbing", "");

    if (isTauri()) {
      const channel = new Channel<CursorScrubEvent>();
      cursorScrubChannelRef.current = channel;
      channel.onmessage = (message) => {
        if (message.type === "move") {
          moveScrub(
            message.deltaX - message.deltaY,
            message.shiftKey,
            message.altKey,
          );
        } else {
          finishScrub();
        }
      };
      const cursorScrub = invoke("begin_cursor_scrub", { channel });
      cursorScrubRef.current = cursorScrub;
      void cursorScrub.catch((cause: unknown) => {
        console.error("Could not begin native number-field scrubbing", cause);
        finishScrub();
      });
    }
  };
};
