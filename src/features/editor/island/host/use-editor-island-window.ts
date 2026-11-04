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
import { useExportIslandStore } from "../../export/export-island-store";
import { toolPanelLabel } from "../../tools/tool-panel-window";
import { EditorKind } from "../../types";

import {
  type EditorIslandKind,
  isStripIsland,
  type PopupPanelIslandContent,
} from "./editor-island-kind";
import {
  editorContent,
  islandContent,
  type IslandWindowPlace,
  islandWindowPlace,
  previewViewport,
} from "./island-window-place";

// The editor's island lives in the editor's panel window, which is laid over
// the editor's content area under its title bar and lets clicks through
// everywhere but the island (`editor-island-panel.tsx`). The picture is
// drawn natively under the page and takes every press over it, so the
// island cannot be part of the page. The window stays up while the editor
// is, so the island comes and goes inside it as what is in hand changes,
// and the window never changes with the island (`island-window-place.ts`);
// it follows the content area as the editor is resized, and the island the
// picture inside it as the editor's layout changes.

const ISLAND_ID = "island";

/** Hands the island window what to show. */
const publish = (panel: string, content: PopupPanelIslandContent) => {
  usePopupPanelStore.getState().open(panel, {
    content,
    focusContents: false,
    id: ISLAND_ID,
  });
};

/** Moves and sizes the island window to `place`. */
const fit = (panel: string, { bounds }: IslandWindowPlace) =>
  movePopupPanel({
    offset: new LogicalPosition(bounds.left, bounds.top),
    panel,
    parentWindowLabel: getCurrentWindow().label,
    size: new LogicalSize(bounds.width, bounds.height),
  }).catch(() => undefined);

/** Shows `island` in the window over the editor's content, or puts it away
 * when `island` is null. `anchorKey` names the strip item a strip island
 * stands over, so the island finds the new one when another is chosen.
 * The Export island, out or showing its finish, stands in for `island`. */
export function useEditorIslandWindow({
  anchorKey = null,
  island: asked,
  workspace,
}: {
  island: EditorIslandKind | null;
  workspace: EditorKind;
  anchorKey?: string | null;
}) {
  const isExportOut = useExportIslandStore(
    (state) => state.isOpen || state.exported !== null,
  );
  const island = isExportOut ? "export" : asked;
  const panel = toolPanelLabel(workspace);
  const shownRef = useRef({ anchor: anchorKey, island });
  useEffect(() => {
    shownRef.current = { anchor: anchorKey, island };
  });

  // What the island holds, and the strip item it stands over, as they
  // change. The window is shown the first time there is an island to hold,
  // and again when it was put away with the editor, as minimising takes it.
  useEffect(() => {
    let disposed = false;
    let frame = 0;
    const settle = async () => {
      const place = islandWindowPlace();
      if (!place) {
        frame = requestAnimationFrame(() => {
          void settle();
        });
        return;
      }
      publish(
        panel,
        islandContent({ anchor: anchorKey, island }, place, workspace),
      );
      if (island === null) return;
      const isShowing = (
        await openPopupPanels().catch((): string[] => [])
      ).includes(panel);
      if (disposed || isShowing) return;
      await showPopupPanel({
        focusContents: false,
        offset: new LogicalPosition(place.bounds.left, place.bounds.top),
        panel,
        parentWindowLabel: getCurrentWindow().label,
        size: new LogicalSize(place.bounds.width, place.bounds.height),
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
      const place = islandWindowPlace();
      if (place) {
        publish(panel, islandContent(shownRef.current, place, workspace));
      }
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

  // The content area moving or changing size, as the editor is resized,
  // takes the window with it; the picture changing inside it, as the strip
  // or a column folds, moves the island alone. The picture's element can
  // arrive after the editor (a recording shows its preparing state first)
  // or be replaced by a new one, which a watch on the old one would never
  // hear of, so it is looked up again whenever the picture or the window
  // changes size, and looked for each frame while there is none, as the
  // island's placing above does. Only a change of the content area's
  // bounds moves the window.
  useEffect(() => {
    if (typeof ResizeObserver === "undefined") return;
    let frame = 0;
    let watched: HTMLElement | null = null;
    let fitted = "";
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
      const place = islandWindowPlace();
      if (!place) {
        schedule();
        return;
      }
      publish(panel, islandContent(shownRef.current, place, workspace));
      const { height, left, top, width } = place.bounds;
      const bounds = [left, top, width, height].join();
      if (bounds !== fitted) {
        fitted = bounds;
        void fit(panel, place);
      }
    };
    const schedule = () => {
      cancelAnimationFrame(frame);
      frame = requestAnimationFrame(follow);
    };
    const observer = new ResizeObserver(schedule);
    const content = editorContent();
    if (content) observer.observe(content);
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
