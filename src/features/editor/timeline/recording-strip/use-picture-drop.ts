// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { getCurrentWebview } from "@tauri-apps/api/webview";
import { useEffect, useRef } from "react";

import { requestImagePlacement } from "../../images/image-requests";
import {
  importLibraryPicture,
  isPicturePath,
} from "../../images/picture-library";
import { setSoundDropRefused } from "../../sound-track/sound-library";
import {
  carryPictureAt,
  expectPlacedPicture,
  pictureLanding,
} from "../annotation-lane/picture-carry";
import { type Playhead } from "../scrub-playhead";

import { dragPoint } from "./drag-point";

import type { ScrubPhase } from "../timeline-seek";

// A picture dragged in from the system, or out of the library, over the
// strip: the annotation lanes come out and carry it as a bar, which says
// where it lands (`annotation-drop-lane.tsx`). Let go there, it becomes an
// image from the moment under the pointer, in the middle of the layer in
// hand, as a pasted picture is placed at the playhead, which goes there
// first; the lanes then put it in the lane it was let go in. Anywhere else
// on the page the drag shows the drop refused; let go on the picture, which
// the picture itself takes, it lands where it is let go
// (`editor/images/dropped.rs`).

/** How long a seek is waited for before the image is placed anyway. */
const SEEK_WAIT_MS = 500;

/** Resolves once the playhead stands at `ratio`, or after a moment. Made
 * with the executor, as the project's library predates
 * `Promise.withResolvers`. */
function arrivedAt(playhead: Playhead, ratio: number) {
  let settle = () => {};
  const promise = new Promise<undefined>((resolve) => {
    settle = () => {
      resolve(undefined);
    };
  });
  const timeout = window.setTimeout(settle, SEEK_WAIT_MS);
  const stop = playhead.subscribe((_, at) => {
    if (Math.abs(at - ratio) > 1e-3) return;
    window.clearTimeout(timeout);
    settle();
  });
  // Listening ends however it settles; `subscribe` may already have been
  // answered, before `stop` was there to call.
  return promise.finally(stop);
}

export function usePictureDrop({
  playhead,
  seek,
  timelineMs,
}: {
  playhead: Playhead;
  /** Moves the playhead to `ratio` of the edited video. */
  seek: (ratio: number, phase: ScrubPhase) => void;
  /** How long the edited video plays, in ms. */
  timelineMs: number;
}) {
  const latestRef = useRef({ playhead, seek, timelineMs });
  latestRef.current = { playhead, seek, timelineMs };
  useEffect(() => {
    let carried: string | null = null;
    let refused = false;
    const showRefused = (next: boolean) => {
      if (next === refused) return;
      refused = next;
      setSoundDropRefused(next).catch((cause: unknown) => {
        console.error("Could not show the drop refused", cause);
      });
    };
    const place = async (path: string, atMs: number) => {
      const {
        playhead: head,
        seek: seekTo,
        timelineMs: total,
      } = latestRef.current;
      const art = await importLibraryPicture(path);
      const ratio = Math.min(Math.max(atMs / Math.max(total, 1), 0), 1);
      const arrived = arrivedAt(head, ratio);
      seekTo(ratio, "start");
      seekTo(ratio, "end");
      await arrived;
      requestImagePlacement("recording", art);
    };
    let stop: (() => void) | undefined;
    let isDisposed = false;
    const follow = (path: string | null, at: { x: number; y: number }) => {
      carried = path;
      if (path === null) return;
      carryPictureAt(at);
      // Where the lanes last said it lands, a step behind the pointer.
      showRefused(pictureLanding() === null);
    };
    const letGo = () => {
      carried = null;
      carryPictureAt(null);
      showRefused(false);
    };
    void getCurrentWebview()
      .onDragDropEvent(({ payload }) => {
        switch (payload.type) {
          case "enter":
            follow(
              payload.paths.find(isPicturePath) ?? null,
              dragPoint(payload.position),
            );
            return;
          case "over":
            follow(carried, dragPoint(payload.position));
            return;
          case "drop": {
            const path = carried;
            const landing = pictureLanding();
            letGo();
            if (path === null || landing === null) return;
            expectPlacedPicture(landing.lanes);
            place(path, landing.atMs).catch((cause: unknown) => {
              expectPlacedPicture(null);
              console.error("Could not place the picture", cause);
            });
            return;
          }
          case "leave":
            letGo();
        }
      })
      .then((unlisten) => {
        if (isDisposed) unlisten();
        else stop = unlisten;
      });
    return () => {
      isDisposed = true;
      stop?.();
    };
  }, []);
}
