// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  getCurrentWindow,
  LogicalPosition,
  LogicalSize,
} from "@tauri-apps/api/window";
import { useEffect, useRef } from "react";

import {
  hidePopupPanel,
  movePopupPanel,
  openPopupPanels,
  showPopupPanel,
} from "../../../popup-panel/api";
import { usePopupPanelStore } from "../../../popup-panel/store";
import { toolPanelLabel } from "../../tool-panels/tool-panel-window";
import { EditorKind } from "../../types";

import {
  type EditorIslandKind,
  isStripIsland,
  type PopupPanelIslandContent,
} from "./editor-island-kind";

// The editor's island lives in the editor's panel window, which is laid over
// the picture, as large as it, and lets clicks through everywhere but the
// island (`editor-island-panel.tsx`). The picture is drawn natively under the
// page and takes every press over it, so the island cannot be part of the
// page. The window stays up while the editor is, so the island comes and
// goes inside it as what is in hand changes, and follows the picture as the
// editor is resized.

const ISLAND_ID = "island";

/** Both workspaces mark their picture area with this; the island stands
 * over it. */
const previewViewport = () =>
  document.querySelector<HTMLElement>("[data-recording-preview-viewport]");

/** How far along the picture, in px from its left edge, the middle of the
 * chosen strip items stands, which their island stands over. Several
 * marked alike, as the strip's gathered items and the parts of one split
 * at a jump are, stand as one, from the first one's start to the last
 * one's end. */
const stripPlace = (viewport: DOMRect) => {
  const marked = [...document.querySelectorAll("[data-strip-island]")].map(
    (element) => element.getBoundingClientRect(),
  );
  if (marked.length === 0) return 0;
  const left = Math.min(...marked.map((bounds) => bounds.left));
  const right = Math.max(...marked.map((bounds) => bounds.right));
  return (left + right) / 2 - viewport.left;
};

const islandContent = (
  island: EditorIslandKind | null,
  viewport: DOMRect,
  workspace: EditorKind,
): PopupPanelIslandContent => ({
  island,
  kind: "island",
  workspace,
  x: isStripIsland(island) ? stripPlace(viewport) : 0,
});

/** Shows `island` in the window over the editor's picture, or puts it away
 * when `island` is null. `anchorKey` names the strip item a strip island
 * stands over, so the island finds the new one when another is chosen. */
export function useEditorIslandWindow({
  anchorKey = null,
  island,
  workspace,
}: {
  island: EditorIslandKind | null;
  workspace: EditorKind;
  anchorKey?: string | null;
}) {
  const panel = toolPanelLabel(workspace);
  const islandRef = useRef(island);
  useEffect(() => {
    islandRef.current = island;
  });

  // What the island holds, and the strip item it stands over, as they
  // change. The window is shown the first time there is an island to hold,
  // and again when it was put away with the editor, as minimising takes it.
  useEffect(() => {
    let disposed = false;
    let frame = 0;
    const settle = async () => {
      const viewport = previewViewport()?.getBoundingClientRect();
      if (!viewport) {
        frame = requestAnimationFrame(() => {
          void settle();
        });
        return;
      }
      usePopupPanelStore.getState().open(panel, {
        content: islandContent(island, viewport, workspace),
        focusContents: false,
        id: ISLAND_ID,
      });
      if (island === null) return;
      const isShowing = (
        await openPopupPanels().catch((): string[] => [])
      ).includes(panel);
      if (disposed || isShowing) return;
      await showPopupPanel({
        focusContents: false,
        offset: new LogicalPosition(viewport.left, viewport.top),
        panel,
        parentWindowLabel: getCurrentWindow().label,
        size: new LogicalSize(viewport.width, viewport.height),
        // The editor keeps working under the island, so a press outside it
        // must not take it away.
        sticky: true,
        triggerId: ISLAND_ID,
      });
    };
    void settle();
    return () => {
      disposed = true;
      cancelAnimationFrame(frame);
    };
  }, [anchorKey, island, panel, workspace]);

  // A strip item's island stands over the item, which moves as the strip
  // scrolls under it, while it plays or as a choice is brought into view.
  const followsScroll = isStripIsland(island);
  useEffect(() => {
    if (!followsScroll) return;
    let frame = 0;
    const follow = () => {
      const viewport = previewViewport()?.getBoundingClientRect();
      if (!viewport) return;
      usePopupPanelStore.getState().open(panel, {
        content: islandContent(islandRef.current, viewport, workspace),
        focusContents: false,
        id: ISLAND_ID,
      });
    };
    const onScroll = () => {
      cancelAnimationFrame(frame);
      frame = requestAnimationFrame(follow);
    };
    document.addEventListener("scroll", onScroll, {
      capture: true,
      passive: true,
    });
    return () => {
      cancelAnimationFrame(frame);
      document.removeEventListener("scroll", onScroll, { capture: true });
    };
  }, [followsScroll, panel, workspace]);

  // The picture moving or changing size, as the editor is resized, takes
  // the window with it. The picture's element can arrive after the editor
  // (a recording shows its preparing state first) or be replaced by a new
  // one, which a watch on the old one would never hear of, so it is looked
  // up again whenever the picture or the window changes size, and looked
  // for each frame while there is none, as the island's placing above does.
  useEffect(() => {
    if (typeof ResizeObserver === "undefined") return;
    let frame = 0;
    let watched: HTMLElement | null = null;
    const follow = () => {
      const viewport = previewViewport();
      if (viewport !== watched) {
        if (watched) observer.unobserve(watched);
        watched = viewport;
        // Observing reports the element's size at once, which follows.
        if (viewport) {
          observer.observe(viewport);
          return;
        }
      }
      if (!viewport) {
        schedule();
        return;
      }
      const bounds = viewport.getBoundingClientRect();
      usePopupPanelStore.getState().open(panel, {
        content: islandContent(islandRef.current, bounds, workspace),
        focusContents: false,
        id: ISLAND_ID,
      });
      void movePopupPanel({
        offset: new LogicalPosition(bounds.left, bounds.top),
        panel,
        parentWindowLabel: getCurrentWindow().label,
        size: new LogicalSize(bounds.width, bounds.height),
      }).catch(() => undefined);
    };
    const schedule = () => {
      cancelAnimationFrame(frame);
      frame = requestAnimationFrame(follow);
    };
    const observer = new ResizeObserver(schedule);
    window.addEventListener("resize", schedule);
    schedule();
    return () => {
      cancelAnimationFrame(frame);
      window.removeEventListener("resize", schedule);
      observer.disconnect();
    };
  }, [panel, workspace]);

  // The editor going takes its island with it.
  useEffect(
    () => () => {
      usePopupPanelStore.getState().close(panel);
      void hidePopupPanel(false, panel);
    },
    [panel],
  );
}
