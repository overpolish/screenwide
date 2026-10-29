// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { getCurrentWindow } from "@tauri-apps/api/window";

import { useTextTemplateFonts } from "../editor/recording/captions/use-text-template-fonts";
import { MomentNotePanel } from "../editor/timeline/moments/moment-note-panel";
import { TextTemplatePanel } from "../editor/tool-panels/text-style/text-template-panel";
import { usePreloadedTemplatePreviews } from "../editor/tool-panels/text-style/use-template-previews";

import { PopupPanelColors } from "./popup-panel-colors";
import { PopupPanelList } from "./popup-panel-list";
import { PopupPanelTool } from "./popup-panel-tool";
import {
  activePopupPanel,
  OpenPopupPanel,
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
  // The font list names each family in its own type, those your templates
  // bring among them.
  useTextTemplateFonts();
  // Only the shared panel window shows the templates.
  usePreloadedTemplatePreviews(panel === SHARED_POPUP_PANEL);

  if (!active) return null;

  // Every panel is drawn in the island's black and its dark dress, over no
  // material or shadow (`tauri.conf.json`), so menus, lists and panels read
  // the same wherever they open. Its corner sits a tight inset out from its
  // items'. Over the island's black it would vanish, so it takes the faint
  // ring the island's angle dial draws, inset so the window still fits its
  // contents.
  return (
    <div className="dark size-full overflow-hidden rounded-panel bg-island text-fg inset-ring inset-ring-fill-strong">
      <PanelContents active={active} panel={panel} />
    </div>
  );
}

function PanelContents({
  active,
  panel,
}: {
  active: OpenPopupPanel;
  panel: string;
}) {
  if (active.content.kind === "tool") {
    return <PopupPanelTool content={active.content} />;
  }

  if (active.content.kind === "note") {
    return <MomentNotePanel content={active.content} />;
  }

  if (active.content.kind === "text-templates") {
    return (
      <TextTemplatePanel
        content={active.content}
        focusContents={active.focusContents}
        id={active.id}
      />
    );
  }

  if (active.content.kind === "colors") {
    return <PopupPanelColors content={active.content} id={active.id} />;
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
