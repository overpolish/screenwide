// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke, isTauri } from "@tauri-apps/api/core";
import { type RefObject, useEffect, useState } from "react";

import { paintScene, PICTURE, sceneFonts, type ToolScene } from "./scene-kit";

import type { PreviewTool } from "./tool-scenes";

// Runs a tool natively beside the page: the app draws the Ruler, OCR or
// Annotate with its own overlay renderer, on or over the screen the page
// paints. The page keeps the layer over its box, clipped to what can be
// seen of it, and moves the tool's pointer. Outside the app there is no
// renderer to run, so nothing opens and the box keeps its painted screen.

type Area = { height: number; width: number; x: number; y: number };

// A page loaded again leaves its previews open in the window, still drawn
// under it, so the first preview a page opens closes them.
let leftovers: Promise<unknown> | null = null;

function closeLeftovers() {
  leftovers ??= invoke("close_tool_previews").catch(() => undefined);
  return leftovers;
}

/** `scene` painted as RGBA in device pixels. */
function picture(scene: ToolScene) {
  const canvas = document.createElement("canvas");
  const context = paintScene(canvas, scene);
  if (!context) return null;
  const pixels = context.getImageData(0, 0, canvas.width, canvas.height);
  return {
    height: canvas.height,
    rgba: new Uint8Array(pixels.data.buffer),
    width: canvas.width,
  };
}

/** The ancestors that clip `element`, found once: the page's layout does
 * not move a box between scroll areas. */
function clippingAncestors(element: HTMLElement) {
  const ancestors: HTMLElement[] = [];
  for (let node = element.parentElement; node; node = node.parentElement) {
    const style = getComputedStyle(node);
    if (style.overflowX !== "visible" || style.overflowY !== "visible") {
      ancestors.push(node);
    }
  }
  return ancestors;
}

function visibleArea(ancestors: HTMLElement[]): Area {
  let left = 0;
  let top = 0;
  let right = window.innerWidth;
  let bottom = window.innerHeight;
  for (const ancestor of ancestors) {
    const rect = ancestor.getBoundingClientRect();
    left = Math.max(left, rect.left);
    top = Math.max(top, rect.top);
    right = Math.min(right, rect.right);
    bottom = Math.min(bottom, rect.bottom);
  }
  return {
    height: Math.max(0, bottom - top),
    width: Math.max(0, right - left),
    x: left,
    y: top,
  };
}

const areaOf = (rect: DOMRect): Area => ({
  height: rect.height,
  width: rect.width,
  x: rect.left,
  y: rect.top,
});

/** The fade of the nearest scroll area the box is in, which a CSS mask
 * cannot carry down to the layer: its area, axis and the px faded before
 * (top or left) and after. */
function edgeFade(ancestors: HTMLElement[]) {
  const viewport = ancestors.find((ancestor) => ancestor.dataset.fadeAxis);
  if (!viewport) return null;
  const start = Number(viewport.dataset.fadeBefore ?? 0);
  const end = Number(viewport.dataset.fadeAfter ?? 0);
  if (start === 0 && end === 0) return null;
  return {
    area: areaOf(viewport.getBoundingClientRect()),
    end,
    start,
    vertical: viewport.dataset.fadeAxis === "vertical",
  };
}

/** Opens `tool` in the box `ref` points at. The picture is painted once, at
 * a size wider than the box can get, and the box shows its middle, so a
 * resize only moves layers. `look` is Annotate's colour and whether its
 * annotations are kept. Answers whether the app is drawing it. */
export function useNativeToolPreview({
  isOn,
  look,
  ref,
  scene,
  tool,
}: {
  isOn: boolean;
  ref: RefObject<HTMLElement | null>;
  scene: ToolScene;
  tool: PreviewTool;
  look?: { color: string; keep: boolean };
}) {
  const [openId, setOpenId] = useState<string | null>(null);
  const color = look?.color;
  const keep = look?.keep ?? false;

  useEffect(() => {
    if (!openId || color === undefined) return;
    void invoke("set_tool_preview_look", { color, id: openId, keep });
  }, [color, keep, openId]);

  useEffect(() => {
    if (!isTauri()) return;
    const id = crypto.randomUUID();
    let closed = false;
    const opened = Promise.all([closeLeftovers(), sceneFonts()]).then(
      async () => {
        const painted = closed ? null : picture(scene);
        if (!painted) return false;
        await invoke("open_tool_preview", painted.rgba, {
          headers: {
            "x-picture-height": String(painted.height),
            "x-picture-width": String(painted.width),
            "x-preview-id": id,
            "x-preview-tool": tool,
            "x-size-height": String(PICTURE.height),
            "x-size-width": String(PICTURE.width),
          },
        });
        return true;
      },
    );
    opened
      .then((open) => {
        if (open && !closed) setOpenId(id);
      })
      .catch((error: unknown) => {
        console.error("The tool preview could not open", error);
      });
    return () => {
      closed = true;
      setOpenId(null);
      // A close sent before the open lands would find nothing to close.
      void opened
        .catch(() => false)
        .then((open) => (open ? invoke("close_tool_preview", { id }) : null));
    };
  }, [scene, tool]);

  // Placement follows the box every frame, so scrolling, resizing and
  // layout shifts all carry the layer with it.
  useEffect(() => {
    const element = ref.current;
    if (!openId || !element) return;
    const ancestors = clippingAncestors(element);
    const radius = parseFloat(getComputedStyle(element).borderTopLeftRadius);
    let frame = 0;
    let placed = "";
    const place = () => {
      const box = areaOf(element.getBoundingClientRect());
      const placement = {
        clip: visibleArea(ancestors),
        fade: edgeFade(ancestors),
        frame: box,
        picture: {
          height: PICTURE.height,
          width: PICTURE.width,
          x: box.x + (box.width - PICTURE.width) / 2,
          y: box.y,
        },
        radius,
        visible: isOn,
      };
      const key = JSON.stringify(placement);
      if (key !== placed) {
        placed = key;
        void invoke("place_tool_preview", { id: openId, placement });
      }
      frame = requestAnimationFrame(place);
    };
    frame = requestAnimationFrame(place);
    return () => {
      cancelAnimationFrame(frame);
    };
  }, [isOn, openId, ref]);

  // The pointer's loop restarts whenever the tool is switched back on, from
  // a fresh session.
  useEffect(() => {
    if (!openId || !isOn) return;
    const send = (input: string, x = 0, y = 0) =>
      void invoke("tool_preview_input", { id: openId, input, x, y });
    let frame = 0;
    let pass = -1;
    let previous = -1;
    let last: { pressed: boolean; x: number; y: number } | null = null;
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
      const pointer = scene.pointer(at, PICTURE.width, PICTURE.height);
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
      frame = requestAnimationFrame(tick);
    };
    frame = requestAnimationFrame(tick);
    return () => {
      cancelAnimationFrame(frame);
    };
  }, [isOn, openId, scene]);

  return openId !== null;
}
