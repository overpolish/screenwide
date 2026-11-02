// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { isTauri } from "@tauri-apps/api/core";
import { type RefObject, useEffect, useRef, useState } from "react";

import { paintPictures } from "./paint-pictures";
import { clippingAncestors, placementOf } from "./preview-placement";
import {
  nativeScene,
  pictureSize,
  sceneOf,
  type SettingsPreviewScene,
} from "./scenes/preview-scenes";
import { type SceneDesktop } from "./scenes/scene-kit";
import {
  closeSettingsPreview,
  openSettingsPreview,
  placeSettingsPreview,
  setSettingsPreviewScene,
} from "./settings-preview-api";
import { useToolPointer } from "./use-tool-pointer";

// A setting's live picture, drawn natively in the box `ref` points at: the
// tool at work with its own renderer, or a recording composed by the
// editor's compositor. The page paints the pictures once, keeps the native
// layer over its box every frame, clipped to what can be seen of it, and
// plays a tool's pointer. A change of the setting's values reaches the
// running scene; another kind of scene opens a new one. Outside the app
// nothing opens and the box keeps its still picture.

export function useSettingsPreview({
  cursorRef,
  desktop: { notch, wallpaper },
  isOn,
  ref,
  scene,
}: {
  /** The pointer's picture, which a tool's scene moves as it plays. */
  cursorRef: RefObject<HTMLElement | null>;
  /** What the scene's desktop is set on (`useDesktopWallpaper`,
   * `useDisplayNotches`). */
  desktop: SceneDesktop;
  isOn: boolean;
  ref: RefObject<HTMLElement | null>;
  scene: SettingsPreviewScene;
}) {
  const [openId, setOpenId] = useState<string | null>(null);
  const sceneRef = useRef(scene);
  sceneRef.current = scene;
  const kind = scene.kind;
  const values = JSON.stringify(scene);

  useEffect(() => {
    if (!isTauri()) return;
    const id = crypto.randomUUID();
    let closed = false;
    const opened = paintPictures(sceneRef.current, { notch, wallpaper }).then(
      async (pictures) => {
        if (closed || !pictures) return false;
        const current = sceneRef.current;
        await openSettingsPreview(id, nativeScene(current), {
          pictures,
          size: pictureSize(current),
        });
        return true;
      },
    );
    opened
      .then((open) => {
        if (open && !closed) setOpenId(id);
      })
      .catch((error: unknown) => {
        console.error("The settings preview could not open", error);
      });
    return () => {
      closed = true;
      setOpenId(null);
      // A close sent before the open lands would find nothing to close.
      void opened
        .catch(() => false)
        .then((open) => (open ? closeSettingsPreview(id) : null));
    };
  }, [kind, notch, wallpaper]);

  useEffect(() => {
    if (!openId) return;
    void setSettingsPreviewScene(openId, nativeScene(sceneRef.current));
  }, [openId, values]);

  // Placement follows the box every frame, so scrolling, resizing and
  // layout shifts all carry the layer with it.
  useEffect(() => {
    const element = ref.current;
    if (!openId || !element) return;
    const ancestors = clippingAncestors(element);
    const definition = sceneOf(sceneRef.current);
    const picture =
      definition.kind === "tool" ? pictureSize(sceneRef.current) : null;
    let frame = 0;
    let placed = "";
    const place = () => {
      const placement = placementOf(element, ancestors, {
        picture,
        visible: isOn,
      });
      const key = JSON.stringify(placement);
      if (key !== placed) {
        placed = key;
        void placeSettingsPreview(openId, placement);
      }
      frame = requestAnimationFrame(place);
    };
    frame = requestAnimationFrame(place);
    return () => {
      cancelAnimationFrame(frame);
    };
  }, [isOn, openId, ref]);

  const definition = sceneOf(scene);
  useToolPointer({
    cursorRef,
    id: openId,
    isOn,
    scene: definition.kind === "tool" ? definition : null,
    valuesRef: sceneRef,
  });

  return openId !== null;
}
