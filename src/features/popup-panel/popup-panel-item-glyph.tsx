// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  ArrowBigDownDash,
  ClipboardCopy,
  Copy,
  Folder,
  FolderOpen,
  FolderRoot,
  FolderSearch,
  HardDrive,
  ImageDown,
  LucideIcon,
  Timer,
  Trash2,
  Undo2,
  X,
} from "lucide-react";

import {
  ArrowToolIcon,
  CounterToolIcon,
  DrawToolIcon,
  HighlightToolIcon,
  ImageToolIcon,
  MagnifyToolIcon,
  RedactToolIcon,
  ShapeToolIcon,
  SpotlightToolIcon,
  TextToolIcon,
} from "../../components/shared/annotation-style/annotation-tool-icons";

import { PopupPanelIcon } from "./store";

// Items cross a window boundary as data, so a glyph travels by name.
const itemGlyphs: Record<PopupPanelIcon, LucideIcon> = {
  clipboard: ClipboardCopy,
  copy: Copy,
  folder: Folder,
  "folder-open": FolderOpen,
  "folder-root": FolderRoot,
  "folder-search": FolderSearch,
  "hard-drive": HardDrive,
  image: ImageDown,
  remove: X,
  restore: Undo2,
  scrolling: ArrowBigDownDash,
  timer: Timer,
  "tool-arrow": ArrowToolIcon,
  "tool-counter": CounterToolIcon,
  "tool-draw": DrawToolIcon,
  "tool-highlight": HighlightToolIcon,
  "tool-image": ImageToolIcon,
  "tool-magnify": MagnifyToolIcon,
  "tool-redact": RedactToolIcon,
  "tool-shape": ShapeToolIcon,
  "tool-spotlight": SpotlightToolIcon,
  "tool-text": TextToolIcon,
  trash: Trash2,
};

export function PopupPanelItemGlyph({ icon }: { icon: PopupPanelIcon }) {
  const Glyph = itemGlyphs[icon];
  return <Glyph />;
}
