// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Layers, LibraryBig } from "lucide-react";
import { AnimatePresence, motion, useReducedMotion } from "motion/react";
import { type ReactNode } from "react";

import { t } from "../../../i18n/i18n";
import { motionDurations, motionEasings } from "../../../lib/motion";
import { cn } from "../../../lib/styling";
import { Toolbar } from "../../../primitives/toolbar/toolbar";
import { ToolbarTool } from "../../../primitives/toolbar/toolbar-tool";
import {
  type CaptureParts,
  type EditorKind,
  type EditorMode,
  editorModeGroups,
  LIBRARY_SHORTCUT,
  TRACKS_SHORTCUT,
} from "../toolbar/editor-modes";

import {
  type EditorIslandToggle,
  useEditorTitleModes,
} from "./editor-title-modes";

// The editor's modes, centred in its title bar between the project's name
// and the window's commands, in groups a gap apart, and after them a
// recording's Tracks and Library, a gap apart as well: each brings out an
// island of its own for the whole timeline. They are tools, not commands, so
// each is its glyph alone, as the column's tools are, naming itself to
// assistive technology and, with its key, in a tooltip; the open one is
// filled as a held toggle is. The group keeps its width, and the name gives
// way first.

export type EditorModeBarProps = {
  kind: EditorKind;
  /** The mode open, or null. */
  mode: EditorMode | null;
  /** Told the mode chosen, or null when the open one is pressed again. */
  onModeChange: (mode: EditorMode | null) => void;
  /** What the capture holds besides its picture, which offers the modes
   * made of each part. */
  parts: CaptureParts;
  /** While a save runs or a sheet stands over the editor. */
  isDisabled?: boolean;
  /** A recording's Library button. */
  library?: EditorIslandToggle | null;
  /** A recording's Tracks button. */
  tracks?: EditorIslandToggle | null;
};

/** The open editor's modes, as its preview publishes them
 * (`usePublishEditorModes`); nothing before it has. */
export function EditorTitleModeBar({ isDisabled }: { isDisabled: boolean }) {
  const modes = useEditorTitleModes((state) => state.modes);
  return modes ? <EditorModeBar {...modes} isDisabled={isDisabled} /> : null;
}

export function EditorModeBar({
  isDisabled = false,
  kind,
  library = null,
  mode,
  onModeChange,
  parts,
  tracks = null,
}: EditorModeBarProps) {
  const groups = editorModeGroups({ kind, parts });
  const reduce = useReducedMotion();
  if (groups.length === 0 && !tracks && !library) return null;
  const transition = reduce
    ? { duration: 0 }
    : { duration: motionDurations.rearrange, ease: motionEasings.out };
  return (
    <Toolbar
      aria-label={t("editor-toolbar-modes")}
      // Each tool keeps its own 1px either side, so tools stand 2px apart
      // and one coming or going takes its spacing with it.
      className="shrink-0 items-center gap-0"
      isDisabled={isDisabled}
    >
      {/* The first render, as the window opens, is drawn as it is. */}
      <AnimatePresence initial={false}>
        {groups.flatMap(({ id: groupId, modes }, index) => [
          ...(index > 0
            ? [<GroupGap key={`gap:${groupId}`} transition={transition} />]
            : []),
          ...modes.map(({ icon: Icon, id, label, shortcut }) => (
            <Presence className="px-px" key={id} transition={transition}>
              <ToolbarTool
                aria-label={label}
                isSelected={mode === id}
                label={label}
                onChange={(isSelected) => {
                  onModeChange(isSelected ? id : null);
                }}
                shortcut={shortcut}
              >
                <Icon />
              </ToolbarTool>
            </Presence>
          )),
        ])}
        {(tracks || library) && groups.length > 0 ? (
          <GroupGap key="gap:tracks" transition={transition} />
        ) : null}
        {tracks ? (
          <Presence className="px-px" key="tracks" transition={transition}>
            <ToolbarTool
              aria-label={t("editor-timeline-tracks-name")}
              isSelected={tracks.isOpen}
              label={t("editor-timeline-tracks")}
              onChange={tracks.onOpenChange}
              shortcut={TRACKS_SHORTCUT}
            >
              <Layers />
            </ToolbarTool>
          </Presence>
        ) : null}
        {library ? (
          <Presence className="px-px" key="library" transition={transition}>
            <ToolbarTool
              aria-label={t("editor-timeline-library-name")}
              isSelected={library.isOpen}
              label={t("editor-timeline-library")}
              onChange={library.onOpenChange}
              shortcut={LIBRARY_SHORTCUT}
            >
              <LibraryBig />
            </ToolbarTool>
          </Presence>
        ) : null}
      </AnimatePresence>
    </Toolbar>
  );
}

type PresenceTransition = { duration: number; ease?: "easeOut" };

/** A tool, or a gap, that grows in from nothing and shrinks back out as
 * the capture comes to have something for it or no longer does, faded as
 * it goes, so the rest slide over rather than jump and the group, centred
 * as it is, stays centred as it changes width. */
function Presence({
  children,
  className,
  transition,
}: {
  children: ReactNode;
  transition: PresenceTransition;
  className?: string;
}) {
  return (
    <motion.div
      animate={{ opacity: 1, width: "auto" }}
      // Clipped with room for a focus ring round what it holds.
      className={cn(
        "flex shrink-0 justify-center overflow-clip [overflow-clip-margin:3px]",
        className,
      )}
      exit={{ opacity: 0, width: 0 }}
      initial={{ opacity: 0, width: 0 }}
      transition={transition}
    >
      {children}
    </motion.div>
  );
}

// Groups stand a gap apart rather than behind rules: the glyphs alone read
// as a row, and a rule between them would be the only line in the title bar.
// The tools either side keep 1px each, which the gap gives back.
function GroupGap({ transition }: { transition: PresenceTransition }) {
  return (
    <Presence transition={transition}>
      <span aria-hidden className="-mx-px w-separate shrink-0" />
    </Presence>
  );
}
