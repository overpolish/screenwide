// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef, useState } from "react";

import { ToolPanelCameraObjects } from "../../tool-panels/tool-panel-store";
import { useModel } from "../microphone/use-model";
import { usePreviewStill } from "../preview/preview-still-store";

import {
  CameraObjectsFollowing,
  CameraPick,
  getRecordingCameraPicks,
  listenToCameraObject,
  listenToCameraObjectsFollowing,
  listenToCameraPicksChanged,
  removeRecordingCameraPick,
  setRecordingCameraPicking,
} from "./camera-picks-api";

export type CameraPicksTool = {
  /** Download the model following objects needs. */
  download: () => void;
  remove: (id: number) => void;
  /** Pick objects on the preview, or stop. */
  setPicking: (isPicking: boolean) => void;
  /** Go to the frame the object was picked in, and pick there with it
   * chosen. */
  show: (id: number) => void;
  shown: ToolPanelCameraObjects;
};

type Held = { artifactId: number; picks: CameraPick[] };

/** Requests for the picks counted as they are made, and the latest whose
 * answer is shown. */
type Order = { asked: number; shown: number };

/** Shows the picks `request` answers with, unless the answer to a later
 * request is shown already: a removal answers once the matte is laid in
 * again, after a pick made meanwhile has answered. */
const inOrder = (
  order: Order,
  request: Promise<CameraPick[]>,
  show: (picks: CameraPick[]) => void,
) => {
  order.asked += 1;
  const ticket = order.asked;
  return request.then((picks) => {
    if (ticket < order.shown) return;
    order.shown = ticket;
    show(picks);
  });
};

/**
 * The objects picked in the camera of the recording `artifactId`, once its
 * matte is made: `isMatted`. Each tells how far following it has got;
 * `progress` is how far laying them into the matte has, as the app tells it. Picking happens on the preview,
 * which draws the camera as recorded and takes a click on it as a pick.
 */
export function useCameraPicks(
  artifactId: number | null,
  isMatted: boolean,
  progress: number | undefined,
): CameraPicksTool | null {
  const model = useModel("cameraObjects");
  const isDownloaded = model.shown?.status === "downloaded";
  const [held, setHeld] = useState<Held | null>(null);
  const [pickingIn, setPickingIn] = useState<number | null>(null);
  const [chosen, setChosen] = useState<number | null>(null);
  const [following, setFollowing] = useState<CameraObjectsFollowing | null>(
    null,
  );
  const orderRef = useRef<Order>({ asked: 0, shown: 0 });
  const isPicking = artifactId !== null && isMatted && pickingIn === artifactId;
  const picks = held?.artifactId === artifactId ? held.picks : [];

  // Picking ends with the recording, or once the matte is gone.
  useEffect(() => {
    if (artifactId === null || !isPicking) return;
    return () => {
      setRecordingCameraPicking(artifactId, false, null).catch(
        (cause: unknown) => {
          console.error("Could not stop picking the camera's objects", cause);
        },
      );
    };
  }, [artifactId, isPicking]);
  // A pick made on the preview chooses that object, and it is let go once
  // the preview leaves the frame it was picked in.
  useEffect(() => {
    if (artifactId === null || !isPicking) return;
    let current = true;
    const stops: (() => void)[] = [];
    const keep = (stop: () => void) => {
      if (current) stops.push(stop);
      else stop();
    };
    const failed = (cause: unknown) => {
      console.error("Could not follow the camera's objects", cause);
    };
    listenToCameraObject("chosen", (object) => {
      if (object.artifactId === artifactId) setChosen(object.id);
    })
      .then(keep)
      .catch(failed);
    listenToCameraObject("left", (object) => {
      if (object.artifactId !== artifactId) return;
      setChosen((chosen) => (chosen === object.id ? null : chosen));
    })
      .then(keep)
      .catch(failed);
    return () => {
      current = false;
      for (const stop of stops) stop();
    };
  }, [artifactId, isPicking]);
  // The preview marks the objects picked, so it is told each time they, as
  // `held`, or the one chosen change.
  useEffect(() => {
    if (artifactId === null || !isPicking) return;
    setRecordingCameraPicking(artifactId, true, chosen).catch(
      (cause: unknown) => {
        console.error("Could not pick the camera's objects", cause);
      },
    );
  }, [artifactId, chosen, held, isPicking]);
  // Asking again once the model arrives starts following what is waiting.
  useEffect(() => {
    if (artifactId === null || !isMatted) return;
    let current = true;
    const read = () => {
      inOrder(orderRef.current, getRecordingCameraPicks(artifactId), (next) => {
        if (current) setHeld({ artifactId, picks: next });
      }).catch((cause: unknown) => {
        console.error("Could not read the camera's objects", cause);
      });
    };
    read();
    let stopFollowing: (() => void) | undefined;
    listenToCameraObjectsFollowing((next) => {
      if (next.artifactId === artifactId) setFollowing(next);
    })
      .then((unlisten) => {
        if (current) stopFollowing = unlisten;
        else unlisten();
      })
      .catch((cause: unknown) => {
        console.error("Could not follow the camera's objects", cause);
      });
    let stop: (() => void) | undefined;
    listenToCameraPicksChanged((id) => {
      if (id === artifactId) read();
    })
      .then((unlisten) => {
        if (current) stop = unlisten;
        else unlisten();
      })
      .catch((cause: unknown) => {
        console.error("Could not follow the camera's objects", cause);
      });
    return () => {
      current = false;
      stop?.();
      stopFollowing?.();
    };
  }, [artifactId, isMatted, isDownloaded]);

  if (artifactId === null || !isMatted) return null;
  // How far an object being followed has got; the rest are followed, or
  // wait their turn.
  const followingOf = (pick: CameraPick) =>
    !pick.followed &&
    following?.artifactId === artifactId &&
    following.ids.includes(pick.id)
      ? following.fraction
      : null;
  return {
    download: model.download,
    remove: (id) => {
      if (chosen === id) setChosen(null);
      inOrder(
        orderRef.current,
        removeRecordingCameraPick(artifactId, id),
        (next) => {
          setHeld({ artifactId, picks: next });
        },
      ).catch((cause: unknown) => {
        console.error("Could not remove the camera's object", cause);
      });
    },
    setPicking: (next) => {
      setPickingIn(next ? artifactId : null);
      setChosen(null);
    },
    show: (id) => {
      const pick = picks.find((each) => each.id === id);
      if (!pick) return;
      usePreviewStill.getState().goTo(pick.frameMs);
      setPickingIn(artifactId);
      setChosen(id);
    },
    shown: {
      chosen: isPicking ? chosen : null,
      isPicking,
      model: model.shown,
      picks: picks.map((pick) => ({
        followed: pick.followed,
        following: followingOf(pick),
        id: pick.id,
        keep: pick.keep,
        painted: pick.line.length > 1,
      })),
      progress: progress ?? 0,
      working: progress !== undefined && progress > 0 && progress < 1,
    },
  };
}
