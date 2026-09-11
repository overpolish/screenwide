// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { PopupPanelList } from "./popup-panel-list";
import { PopupPanelTool } from "./popup-panel-tool";
import { usePopupPanelStore } from "./store";

/**
 * The one panel window, showing whatever was last opened in it.
 *
 * It is a host and nothing more: what the panel holds - a list of choices or
 * an editor tool's controls - is decided by the window that opened it.
 */
export function PopupPanelWindow() {
  const active = usePopupPanelStore((state) => state.active);

  if (!active) return null;

  if (active.content.kind === "tool") {
    return <PopupPanelTool content={active.content} />;
  }

  return (
    <PopupPanelList
      content={active.content}
      focusContents={active.focusContents}
      id={active.id}
      label={active.label}
    />
  );
}
