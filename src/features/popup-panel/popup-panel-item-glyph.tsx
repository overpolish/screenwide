// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  ArrowBigDownDash,
  Clapperboard,
  ClipboardCopy,
  Copy,
  Eye,
  EyeOff,
  Folder,
  FolderOpen,
  FolderRoot,
  FolderSearch,
  HardDrive,
  ImageDown,
  Import,
  Link2,
  LucideIcon,
  PenLine,
  SeparatorVertical,
  ShieldCheck,
  Share,
  Sparkle,
  SpellCheck,
  Timer,
  Trash2,
  Undo2,
  X,
} from "lucide-react";

import { cn } from "../../lib/styling";
import {
  ArrowToolIcon,
  CounterToolIcon,
  DrawToolIcon,
  FocusToolIcon,
  HighlightToolIcon,
  ImageToolIcon,
  MagnifyToolIcon,
  RedactToolIcon,
  ShapeToolIcon,
  SpotlightToolIcon,
  TextToolIcon,
} from "../annotations/annotation-tool-icons";

import type { AnnotationKind } from "../annotations/types";

// Items cross a window boundary as data, so a glyph travels by name. Every
// annotation tool has one, under `tool-` so it stays apart from `image`, the
// export-an-image glyph.
const itemGlyphs = {
  allow: ShieldCheck,
  "caption-join": Link2,
  "caption-split": SeparatorVertical,
  clipboard: ClipboardCopy,
  copy: Copy,
  edit: PenLine,
  export: Share,
  folder: Folder,
  "folder-open": FolderOpen,
  "folder-root": FolderRoot,
  "folder-search": FolderSearch,
  "hard-drive": HardDrive,
  hide: EyeOff,
  image: ImageDown,
  import: Import,
  "key-word": Sparkle,
  remove: X,
  restore: Undo2,
  scene: Clapperboard,
  scrolling: ArrowBigDownDash,
  show: Eye,
  spelling: SpellCheck,
  timer: Timer,
  "tool-arrow": ArrowToolIcon,
  "tool-counter": CounterToolIcon,
  "tool-draw": DrawToolIcon,
  "tool-focus": FocusToolIcon,
  "tool-highlight": HighlightToolIcon,
  "tool-image": ImageToolIcon,
  "tool-magnify": MagnifyToolIcon,
  "tool-redact": RedactToolIcon,
  "tool-shape": ShapeToolIcon,
  "tool-spotlight": SpotlightToolIcon,
  "tool-text": TextToolIcon,
  trash: Trash2,
} satisfies Record<`tool-${AnnotationKind}`, LucideIcon> &
  Record<string, LucideIcon>;

/** A glyph a pop-up item can carry, by its name in the registry. */
export type PopupPanelIcon = keyof typeof itemGlyphs;

/** Glyphs that point the way text runs, and turn with it. */
const directionalGlyphs = new Set<PopupPanelIcon>(["restore"]);

export function PopupPanelItemGlyph({
  className,
  icon,
}: {
  icon: PopupPanelIcon;
  className?: string;
}) {
  const Glyph = itemGlyphs[icon];
  return (
    <Glyph
      className={cn(
        directionalGlyphs.has(icon) && "rtl:-scale-x-100",
        className,
      )}
    />
  );
}
