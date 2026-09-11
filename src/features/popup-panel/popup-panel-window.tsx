// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { getCurrentWindow } from "@tauri-apps/api/window";

import { PopupPanelList } from "./popup-panel-list";
import { PopupPanelTool } from "./popup-panel-tool";
import {
  activePopupPanel,
  SHARED_POPUP_PANEL,
  usePopupPanelStore,
} from "./store";

let resolvedPanel: string | undefined;

/**
 * Which panel window this is, read off its own label: the shared listbox, or
 * one of the editors' tool panels. They are the same page, so the window it
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
 * an editor tool's controls - is decided by the window that opened it.
 */
export function PopupPanelWindow() {
  const panel = currentPanelLabel();
  const active = usePopupPanelStore((state) => activePopupPanel(state, panel));

  if (!active) return null;

  if (active.content.kind === "tool") {
    return <PopupPanelTool content={active.content} />;
  }

  return (
    <PopupPanelList
      content={active.content}
      focusContents={active.focusContents}
      id={active.id}
      label={active.label ?? ""}
      panel={panel}
    />
  );
}
