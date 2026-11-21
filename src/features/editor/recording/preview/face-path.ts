// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useMemo, useSyncExternalStore } from "react";

import { ownCameraPicture } from "../scenes/recording-scene-placement";

import type { CameraOverlaySettings } from "../../types";
import type { Face } from "../scenes/recording-scene-framing";

/**
 * The face the preview follows over the camera, read once from it and again
 * when it is found (`face_preview.rs`). A camera that follows the face is
 * placed by it, so the editor's twin of the preview's layout
 * (`recording-scene-placement.ts`) reads the face here and puts the camera,
 * its crop window and its selection where the preview draws them, as it does
 * the screen's from the clips alone.
 */
type FacePath = { points: [number, number][]; stepMs: number };

const store: {
  listeners: Set<() => void>;
  path: FacePath | null;
  positionMs: number;
} = { listeners: new Set(), path: null, positionMs: 0 };

const subscribe = (listener: () => void) => {
  store.listeners.add(listener);
  return () => {
    store.listeners.delete(listener);
  };
};

const publish = (path: FacePath | null) => {
  store.path = path;
  store.listeners.forEach((listener) => {
    listener();
  });
};

/** Where the face is centred `ms` into the camera, held at either end. The
 * twin of `FacePath::at`. */
export const faceAt = (path: FacePath, ms: number): Face | null => {
  const last = path.points.length - 1;
  if (last < 0) return null;
  const at = Math.max(0, ms / path.stepMs);
  const below = Math.min(Math.floor(at), last);
  const above = Math.min(below + 1, last);
  const share = Math.min(1, Math.max(0, at - below));
  const [ax, ay] = path.points[below];
  const [bx, by] = path.points[above];
  return { x: ax + (bx - ax) * share, y: ay + (by - ay) * share };
};

/** The face path, once it is found. */
export const useFacePath = () =>
  useSyncExternalStore(subscribe, () => store.path);

/** The face at the paused playhead, for an edit made outside a render. */
export const faceNow = (): Face | null =>
  store.path ? faceAt(store.path, store.positionMs) : null;

/** The recording's own camera `overlay` with its picture where the preview
 * draws it at `positionMs`: following the face, about the face. A camera of
 * `source` size; unchanged without one or before the face is found. */
export function useFollowedOverlay(
  overlay: CameraOverlaySettings,
  source: { height: number; width: number } | null | undefined,
  positionMs: number,
) {
  const path = useFacePath();
  return useMemo(() => {
    const face = path ? faceAt(path, positionMs) : null;
    if (overlay.followFace !== true || !face || !source || source.height <= 0)
      return overlay;
    const picture = ownCameraPicture({
      aspect: source.width / source.height,
      face,
      overlay,
    });
    return {
      ...overlay,
      cameraWidth: picture.width,
      cameraX: picture.x,
      cameraY: picture.y,
    };
  }, [overlay, path, positionMs, source]);
}
/** Reads the face path from the preview of the session `sessionId`, again
 * whenever it is found anew, and keeps the paused playhead `positionMs`,
 * which `faceNow` reads the face at. */
export function useFacePathLoader(
  sessionId: number | null,
  positionMs: number,
) {
  useEffect(() => {
    store.positionMs = positionMs;
  }, [positionMs]);
  useEffect(() => {
    if (sessionId === null) return;
    let isCurrent = true;
    const load = () => {
      void invoke<FacePath | null>("recording_preview_face_path")
        .then((path) => {
          if (isCurrent) publish(path);
        })
        .catch(() => undefined);
    };
    load();
    const unlisten = listen("preview://face-path", load);
    return () => {
      isCurrent = false;
      void unlisten.then((stop) => {
        stop();
      });
    };
  }, [sessionId]);
}
