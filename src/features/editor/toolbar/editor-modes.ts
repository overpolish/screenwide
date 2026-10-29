// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  Captions,
  Keyboard,
  LayoutPanelLeft,
  type LucideIcon,
  Monitor,
  MousePointerClick,
  ScrollText,
  Video,
  Volume2,
} from "lucide-react";

import { t } from "../../../i18n/i18n";
import { FrameToolIcon } from "../glyphs/frame-tool-icon";

// The modes the tool column opens, each named and drawn as the island it
// opens is titled, so the button and the island read as one. A mode works
// on the whole recording rather than on something chosen in it.

export type EditorKind = "audio" | "recording" | "screenshot";

export type EditorMode =
  | "camera"
  | "captions"
  | "cursor"
  | "frame"
  | "keyboard"
  | "scene"
  | "screen"
  | "sound"
  | "transcript";

export type EditorModeButton = {
  icon: LucideIcon;
  id: EditorMode;
  label: string;
  /** The key that opens it, in the hotkey format tooltips read
   * (`Shift+KeyC`), as the editor's key map has it. */
  shortcut?: string;
};

/** The modes offered for a kind of capture, in groups that the column
 * divides: the picture (its frame, the screen, the camera, the scene), what
 * was done while recording (the cursor, the keys), then what was said and
 * heard. The camera is offered only where the recording has one. A capture
 * without a picture has nothing to show captions over, so it offers only
 * its transcript and sound. */
export function editorModeGroups({
  hasCamera,
  kind,
}: {
  hasCamera: boolean;
  kind: EditorKind;
}): EditorModeButton[][] {
  const frame: EditorModeButton = {
    icon: FrameToolIcon,
    id: "frame",
    label: t("editor-tool-frame"),
    shortcut: "F",
  };
  const screen: EditorModeButton = {
    icon: Monitor,
    id: "screen",
    label:
      kind === "screenshot"
        ? t("editor-island-screenshot")
        : t("editor-island-screen"),
    shortcut: "C",
  };
  const captions: EditorModeButton = {
    icon: Captions,
    id: "captions",
    label: t("editor-island-captions"),
    shortcut: "U",
  };
  const heard: EditorModeButton[] = [
    {
      icon: ScrollText,
      id: "transcript",
      label: t("editor-toolbar-transcript"),
      shortcut: "Shift+KeyT",
    },
    { icon: Volume2, id: "sound", label: t("editor-island-sound") },
  ];
  switch (kind) {
    case "audio":
      return [heard];
    case "screenshot":
      return [[frame, screen]];
    case "recording": {
      const camera: EditorModeButton = {
        icon: Video,
        id: "camera",
        label: t("editor-island-camera"),
      };
      return [
        [
          frame,
          screen,
          ...(hasCamera ? [camera] : []),
          {
            icon: LayoutPanelLeft,
            id: "scene",
            label: t("editor-island-scene"),
            shortcut: "L",
          },
        ],
        [
          {
            icon: MousePointerClick,
            id: "cursor",
            label: t("editor-panels-cursor"),
            shortcut: "Shift+KeyC",
          },
          {
            icon: Keyboard,
            id: "keyboard",
            label: t("editor-panels-keyboard"),
            shortcut: "K",
          },
        ],
        [captions, ...heard],
      ];
    }
  }
}
