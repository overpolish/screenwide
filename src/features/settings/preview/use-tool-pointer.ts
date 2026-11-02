// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type RefObject, useEffect } from "react";

import {
  nativeScene,
  type SettingsPreviewScene,
} from "./scenes/preview-scenes";
import { TOOL_PICTURE, type Pointer, type ToolScene } from "./scenes/scene-kit";
import { sendSettingsPreviewInput } from "./settings-preview-api";

import type { PreviewInput } from "../../../bindings/PreviewInput";

// Plays a tool scene's pointer into the open preview `id`, a pass at a
// time, each pass from a fresh session, and moves the pointer's picture
// with it: the tools draw what the pointer does, not the pointer, which the
// system draws over them. It restarts whenever the tool is switched back
// on; a change of the setting's values reaches the pass under way.

export function useToolPointer({
  cursorRef,
  id,
  isOn,
  scene,
  valuesRef,
}: {
  /** The pointer's picture, moved to where the pointer is and hidden while
   * it is away. */
  cursorRef: RefObject<HTMLElement | null>;
  id: string | null;
  isOn: boolean;
  scene: ToolScene | null;
  /** The setting's values the scene is played with, such as where
   * Annotate's counter points. */
  valuesRef: RefObject<SettingsPreviewScene>;
}) {
  useEffect(() => {
    if (!id || !scene || !isOn) return;
    const cursor = cursorRef.current;
    const send = (input: PreviewInput, x = 0, y = 0) =>
      void sendSettingsPreviewInput(id, input, { x, y });
    let frame = 0;
    let pass = -1;
    let previous = -1;
    let last: Pointer = null;
    const start = performance.now();
    const tick = (now: number) => {
      const seconds = Math.max(0, (now - start) / 1000);
      const at = seconds % scene.duration;
      const thisPass = Math.floor(seconds / scene.duration);
      if (thisPass !== pass) {
        pass = thisPass;
        previous = -1;
        last = null;
        send("reset");
      }
      for (const cue of scene.cues ?? []) {
        if (previous < cue.at && cue.at <= at) send(cue.input);
      }
      previous = at;
      const pointer = scene.pointer(
        at,
        TOOL_PICTURE,
        nativeScene(valuesRef.current),
      );
      if (pointer) {
        const moved = !last || last.x !== pointer.x || last.y !== pointer.y;
        if (pointer.pressed && !last?.pressed)
          send("down", pointer.x, pointer.y);
        else if (!pointer.pressed && last?.pressed)
          send("up", pointer.x, pointer.y);
        else if (moved)
          send(pointer.pressed ? "drag" : "hover", pointer.x, pointer.y);
      } else if (last?.pressed) {
        send("up", last.x, last.y);
      }
      last = pointer;
      if (cursor) {
        cursor.style.visibility = pointer ? "visible" : "hidden";
        if (pointer)
          cursor.style.transform = `translate(${String(pointer.x)}px, ${String(pointer.y)}px)`;
      }
      frame = requestAnimationFrame(tick);
    };
    frame = requestAnimationFrame(tick);
    return () => {
      cancelAnimationFrame(frame);
      if (cursor) cursor.style.visibility = "hidden";
    };
  }, [cursorRef, id, isOn, scene, valuesRef]);
}
