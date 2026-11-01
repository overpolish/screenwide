// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { getCurrentWindow } from "@tauri-apps/api/window";

import { EditorIslandPanel } from "../editor/island/host/editor-island-panel";
import { useTextTemplateFonts } from "../editor/recording/captions/use-text-template-fonts";

import { PopupPanelColors } from "./popup-panel-colors";
import { PopupPanelList } from "./popup-panel-list";
import {
  activePopupPanel,
  OpenPopupPanel,
  PopupPanelContent,
  SHARED_POPUP_PANEL,
  usePopupPanelStore,
} from "./store";

import type { PopupPanelIslandContent } from "../editor/island/host/editor-island-kind";

let resolvedPanel: string | undefined;

/**
 * Which panel window this is, read off its own label: the shared listbox, or
 * one of the editors' island panels. They are the same page, so the window it
 * runs in is what says whose entry to show. Memoized because a window's label
 * never changes; outside the app - a story - it is the shared listbox.
 */
function currentPanelLabel(): string {
  resolvedPanel ??= (() => {
    try {
      return getCurrentWindow().label;
    } catch {
      return SHARED_POPUP_PANEL;
    }
  })();

  return resolvedPanel;
}

/**
 * One panel window, showing whatever was last opened in it.
 *
 * It is a host and nothing more: what the panel holds - a list of choices or
 * a picker - is decided by the window that opened it.
 */
export function PopupPanelWindow() {
  const panel = currentPanelLabel();
  const active = usePopupPanelStore((state) => activePopupPanel(state, panel));
  // The font list names each family in its own type, those your templates
  // bring among them.
  useTextTemplateFonts();

  if (!active) return null;

  // The island draws its own black, at its own shape, over a window as
  // large as the picture it stands over.
  if (active.content.kind === "island")
    return <EditorIslandPanel content={active.content} />;

  // Every panel is drawn in the island's black and its dark dress, over no
  // material or shadow (`tauri.conf.json`), so menus, lists and panels read
  // the same wherever they open. Its corner sits a tight inset out from its
  // items'. Over the island's black it would vanish, so it takes the faint
  // ring the island's angle dial draws, inset so the window still fits its
  // contents.
  return (
    <div className="dark size-full overflow-hidden rounded-panel bg-island text-fg inset-ring inset-ring-fill-strong">
      <PanelContents active={active} content={active.content} panel={panel} />
    </div>
  );
}

function PanelContents({
  active,
  content,
  panel,
}: {
  active: OpenPopupPanel;
  content: Exclude<PopupPanelContent, PopupPanelIslandContent>;
  panel: string;
}) {
  if (content.kind === "colors") {
    return <PopupPanelColors content={content} id={active.id} />;
  }

  return (
    <PopupPanelList
      content={content}
      focusContents={active.focusContents}
      id={active.id}
      label={active.label ?? ""}
      panel={panel}
    />
  );
}
