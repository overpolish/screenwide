// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Channel, invoke, isTauri } from "@tauri-apps/api/core";
import {
  PointerEvent as ReactPointerEvent,
  useCallback,
  useEffect,
  useRef,
} from "react";

import { RecordingTimelineTrimEdge } from "../recording-timeline-edit";

import type { TimelineBladeController } from "./timeline-blade";

const isMacOS = navigator.userAgent.includes("Mac");
type CursorScrubEvent = { deltaX: number; type: "move" } | { type: "end" };

export function useTimelineNativeTrim({
  blade,
  outputPositionAt,
}: {
  blade: TimelineBladeController;
  outputPositionAt: (clientX: number) => number;
}) {
  const channelRef = useRef<Channel<CursorScrubEvent> | null>(null);
  const nativeScrubRef = useRef<Promise<unknown> | null>(null);
  const dragRef = useRef<{
    anchorClientX: number;
    edge: RecordingTimelineTrimEdge;
    element: HTMLSpanElement;
    outputPosition: number;
    travel: number;
  } | null>(null);
  const position = useCallback(() => {
    const drag = dragRef.current;
    return drag
      ? drag.outputPosition +
          outputPositionAt(drag.anchorClientX + drag.travel) -
          outputPositionAt(drag.anchorClientX)
      : 0;
  }, [outputPositionAt]);
  const releaseNative = useCallback((cursorOffsetX = 0) => {
    document.documentElement.removeAttribute("data-timeline-trimming");
    channelRef.current = null;
    const nativeScrub = nativeScrubRef.current;
    nativeScrubRef.current = null;
    if (nativeScrub)
      void nativeScrub
        .catch(() => undefined)
        .then(() => invoke("end_cursor_scrub", { cursorOffsetX }))
        .catch(() => undefined);
  }, []);
  const finish = useCallback(() => {
    const drag = dragRef.current;
    if (!drag) return;
    blade.endTrim(position());
    dragRef.current = null;
    requestAnimationFrame(() => {
      const bounds = drag.element.getBoundingClientRect();
      const boundaryX = drag.edge === "start" ? bounds.left : bounds.right;
      releaseNative(boundaryX - drag.anchorClientX);
    });
  }, [blade, position, releaseNative]);
  const move = useCallback(
    (deltaX: number) => {
      const drag = dragRef.current;
      if (!drag) return;
      drag.travel += deltaX;
      blade.updateTrim(position());
    },
    [blade, position],
  );
  useEffect(() => {
    if (isTauri() && !isMacOS) return;
    const onMove = (event: PointerEvent) => {
      move(event.movementX);
    };
    document.addEventListener("pointermove", onMove);
    document.addEventListener("pointerup", finish);
    document.addEventListener("pointercancel", finish);
    window.addEventListener("blur", finish);
    return () => {
      document.removeEventListener("pointermove", onMove);
      document.removeEventListener("pointerup", finish);
      document.removeEventListener("pointercancel", finish);
      window.removeEventListener("blur", finish);
    };
  }, [finish, move]);
  useEffect(() => releaseNative, [releaseNative]);

  return (
    segmentId: number,
    edge: RecordingTimelineTrimEdge,
    event: ReactPointerEvent<HTMLSpanElement>,
  ) => {
    if (event.button !== 0) return;
    event.preventDefault();
    event.stopPropagation();
    const outputPosition = outputPositionAt(event.clientX);
    dragRef.current = {
      anchorClientX: event.clientX,
      edge,
      element: event.currentTarget,
      outputPosition,
      travel: 0,
    };
    blade.beginTrim(segmentId, edge, outputPosition);
    document.documentElement.setAttribute("data-timeline-trimming", "");
    if (!isTauri()) return;
    const channel = new Channel<CursorScrubEvent>();
    channelRef.current = channel;
    channel.onmessage = (message) => {
      if (message.type === "move") move(message.deltaX);
      else finish();
    };
    const nativeScrub = invoke("begin_cursor_scrub", { channel });
    nativeScrubRef.current = nativeScrub;
    void nativeScrub.catch((cause: unknown) => {
      console.error("Could not begin native timeline trimming", cause);
      finish();
    });
  };
}
