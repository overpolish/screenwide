// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Clock, Folder, FolderRoot, HardDrive, Trash2 } from "lucide-react";
import { AnimatePresence, LayoutGroup } from "motion/react";
import { useEffect, useRef, useState } from "react";

import { SidebarNav } from "../../../composites/sidebar-nav/sidebar-nav";
import { WindowHeader } from "../../../composites/window-header/window-header";
import { WindowShell } from "../../../composites/window-shell/window-shell";
import { SearchField } from "../../../primitives/input-fields/search-field";
import { ScrollArea } from "../../../primitives/scroll-area/scroll-area";
import { Text } from "../../../primitives/text/text";

import { EditorStepIn } from "./editor-step-in";
import { PROJECTS } from "./library";
import { LibraryView } from "./library-view";
import { searchMoments } from "./moment-search";
import { Peek } from "./peek";
import { leftAt, type Project } from "./projects-model";
import { SearchResults } from "./search-results";
import { StorageStrip } from "./storage-strip";

// The project browser, reworked so projects are browsed by what happened in
// them. A card moves through its recording's steps under the pointer;
// search finds moments (what was said, clicked, typed, switched to) rather
// than files; pressing a card peeks at it in place, playing, and Return
// grows it into the editor. Recent opens on what was left part way through,
// takes of one thing stack, and the disk the library takes is a strip that
// points at what takes it. Space peeks at the card under the pointer.

const SOURCES = [
  { icon: <Clock />, id: "Recent", label: "Recent" },
  { icon: <FolderRoot />, id: "Screenwide", label: "Screenwide" },
  { icon: <Folder />, id: "Tutorials", label: "Tutorials" },
  { icon: <HardDrive />, id: "Client work", label: "Client work" },
  {
    icon: <Trash2 />,
    id: "Recently Deleted",
    isPinned: true,
    label: "Recently Deleted",
  },
];

type Moment = { at: number; id: string };

export type ProjectsWindowProps = {
  initialPeek?: Moment;
  initialQuery?: string;
  initialReclaiming?: boolean;
  initialTakesOpen?: boolean;
};

export function ProjectsWindow({
  initialPeek,
  initialQuery = "",
  initialReclaiming = false,
  initialTakesOpen = false,
}: ProjectsWindowProps) {
  const [library, setLibrary] = useState(PROJECTS);
  const [source, setSource] = useState("Recent");
  const [query, setQuery] = useState(initialQuery);
  const [peek, setPeek] = useState<Moment | null>(initialPeek ?? null);
  const [opened, setOpened] = useState<Moment | null>(null);
  const [linkedId, setLinkedId] = useState<string | null>(null);
  const [isReclaiming, setIsReclaiming] = useState(initialReclaiming);
  const [openTakes, setOpenTakes] = useState<ReadonlySet<string>>(
    () => new Set(initialTakesOpen ? ["settings"] : []),
  );
  const hoveredRef = useRef<string | null>(null);
  // The project whose picture the peek or the editor has taken from its
  // card, so only one holds the shared picture at a time.
  const takenId = peek?.id ?? opened?.id ?? null;

  const listed =
    source === "Recent"
      ? library
      : library.filter(({ location }) => location === source);
  const byId = (id: string | undefined) =>
    library.find((project) => project.id === id);
  const peeked = byId(peek?.id);
  const openedProject = byId(opened?.id);
  const onPeek = (project: Project, at: number) => {
    setPeek({ at, id: project.id });
  };
  const onOpen = (project: Project, at: number) => {
    setPeek(null);
    setOpened({ at, id: project.id });
  };

  // Space looks at the card under the pointer, as Finder's Quick Look does.
  const spaceRef = useRef<(event: KeyboardEvent) => void>(() => undefined);
  useEffect(() => {
    spaceRef.current = (event) => {
      if (event.key !== " " || peek || opened) return;
      if ((event.target as Element).closest("input, textarea")) return;
      const project = byId(hoveredRef.current ?? undefined);
      if (!project) return;
      event.preventDefault();
      onPeek(project, leftAt(project) ?? project.steps[0].start);
    };
  });
  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      spaceRef.current(event);
    };
    window.addEventListener("keydown", onKey);
    return () => {
      window.removeEventListener("keydown", onKey);
    };
  }, []);

  const title = query.trim() ? "Search" : source;
  const content = query.trim() ? (
    <SearchResults
      onOpen={onOpen}
      onPeek={onPeek}
      peekedId={takenId}
      query={query}
      results={searchMoments(listed, query)}
    />
  ) : source === "Recently Deleted" ? (
    <div className="flex grow flex-col items-center justify-center gap-1 py-24 text-center">
      <Text variant="heading">No deleted projects</Text>
      <Text className="text-fg-secondary" variant="body">
        Projects you delete wait here for 30 days.
      </Text>
    </div>
  ) : (
    <LibraryView
      isReclaiming={isReclaiming}
      linkedId={linkedId}
      onFree={(freed) => {
        setLibrary((current) =>
          current.filter((project) => !freed.includes(project)),
        );
        setIsReclaiming(false);
      }}
      onHoverChange={(id) => {
        hoveredRef.current = id;
      }}
      onKeepTake={(kept) => {
        setLibrary((current) =>
          current.filter(
            (project) =>
              project.take?.group !== kept.take?.group || project === kept,
          ),
        );
      }}
      onOpen={onOpen}
      onPeek={onPeek}
      onToggleTakes={(group) => {
        setOpenTakes((current) => {
          const next = new Set(current);
          if (!next.delete(group)) next.add(group);
          return next;
        });
      }}
      openTakes={openTakes}
      peekedId={takenId}
      projects={listed}
      showsShelf={source === "Recent"}
    />
  );

  return (
    <LayoutGroup>
      <div className="relative size-full">
        <WindowShell
          header={
            <WindowHeader
              actions={
                <SearchField
                  aria-label="Search projects"
                  className="w-72"
                  onChange={setQuery}
                  placeholder="Search what was said or done"
                  value={query}
                />
              }
              title={title}
            />
          }
        >
          <div className="flex min-h-0 grow gap-group ps-inset">
            <div className="flex w-48 shrink-0 flex-col gap-separate pb-inset">
              <SidebarNav
                aria-label="Project locations"
                className="grow"
                isExpandable={false}
                isExpanded
                items={SOURCES}
                onSelectionChange={setSource}
                selected={source}
              />
              <StorageStrip
                isReclaiming={isReclaiming}
                linkedId={linkedId}
                onLink={setLinkedId}
                onReclaim={() => {
                  setQuery("");
                  setSource("Recent");
                  setIsReclaiming(!isReclaiming);
                }}
                projects={library}
              />
            </div>
            <ScrollArea inWindowCorner rootClassName="min-h-0 min-w-0 grow">
              <div
                className="flex flex-col gap-group pe-inset pt-1 pb-inset"
                key={`${source}-${String(isReclaiming)}`}
              >
                {content}
              </div>
            </ScrollArea>
          </div>
        </WindowShell>
        <AnimatePresence>
          {peeked && peek ? (
            <Peek
              from={peek.at}
              key={peeked.id}
              onClose={() => {
                setPeek(null);
              }}
              onOpen={(at) => {
                onOpen(peeked, at);
              }}
              project={peeked}
            />
          ) : null}
        </AnimatePresence>
        <AnimatePresence>
          {openedProject && opened ? (
            <EditorStepIn
              at={opened.at}
              key={openedProject.id}
              onBack={() => {
                setOpened(null);
              }}
              project={openedProject}
            />
          ) : null}
        </AnimatePresence>
      </div>
    </LayoutGroup>
  );
}
