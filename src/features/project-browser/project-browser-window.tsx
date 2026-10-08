// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { convertFileSrc } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect, useRef, useState } from "react";

import {
  addProjectLocation,
  deleteProject,
  duplicateProjects,
  emptyRecentlyDeleted,
  forgetRecentProject,
  getProjectLocations,
  getProjectThumbnail,
  hideProjectBrowser,
  listProjects,
  moveProjects,
  openOtherProject,
  openProjectFile,
  openProjectLocation,
  PROJECT_STILL_EVENT,
  PROJECTS_CHANGED_EVENT,
  removeProjectLocation,
  renameProject,
  restoreProjects,
  revealProject,
  trashProjects,
} from "./api";
import { ProjectBrowser } from "./project-browser";
import {
  RECENT,
  sourceFor,
  type ProjectLocation,
  type ProjectSummary,
} from "./types";
import { useProjectCopy } from "./use-project-copy";
import { useScrubStrips } from "./use-scrub-strips";
import { useUndoDelete } from "./use-undo-delete";

const message = (cause: unknown) =>
  cause instanceof Error ? cause.message : String(cause);

/** The project browser, kept in step with what is on disk. */
export function ProjectBrowserWindow() {
  const [locations, setLocations] = useState<ProjectLocation[]>([]);
  const [selected, setSelected] = useState(RECENT);
  const [projects, setProjects] = useState<ProjectSummary[] | null>(null);
  const [thumbnails, setThumbnails] = useState<Record<string, string | null>>(
    {},
  );
  const [error, setError] = useState<string | null>(null);
  // Bumped to read everything again: after an action, a change Rust reports,
  // or a return to the window, since projects can change outside the app.
  const [generation, setGeneration] = useState(0);
  // Stills already asked for, by manifest and edit time, so a refresh asks
  // again only for a project whose movie may have changed.
  const requestedRef = useRef(new Set<string>());

  useEffect(() => {
    // A read for a selection the user has since left is dropped.
    let current = true;
    Promise.all([getProjectLocations(), listProjects(sourceFor(selected))])
      .then(([nextLocations, nextProjects]) => {
        if (!current) return;
        setLocations(nextLocations);
        // A location whose drive has gone cannot be shown, so the window
        // goes to the projects folder, or to Recent while that is missing
        // too.
        if (
          nextLocations.some(
            ({ available, path }) => path === selected && !available,
          )
        ) {
          const home = nextLocations.find(
            ({ available, isDefault }) => isDefault && available,
          );
          setProjects(null);
          setSelected(home?.path ?? RECENT);
          return;
        }
        setProjects(nextProjects);
      })
      .catch((cause: unknown) => {
        if (current) setError(message(cause));
      });
    return () => {
      current = false;
    };
  }, [generation, selected]);

  // Each fetch gets its own address: the editor redraws a still in place,
  // and the webview would otherwise keep showing the copy it has.
  const loadThumbnail = (file: string, key?: string) => {
    getProjectThumbnail(file)
      .then((path) => {
        setThumbnails((current) => ({
          ...current,
          [file]: path
            ? `${convertFileSrc(path)}?v=${String(Date.now())}`
            : null,
        }));
      })
      .catch(() => {
        if (key) requestedRef.current.delete(key);
      });
  };
  const loadThumbnailRef = useRef(loadThumbnail);
  loadThumbnailRef.current = loadThumbnail;
  const forgetStripRef = useRef<(file: string) => void>(() => undefined);

  useEffect(() => {
    const reload = () => {
      setGeneration((value) => value + 1);
    };
    const subscriptions = [
      listen(PROJECTS_CHANGED_EVENT, reload),
      listen<string>(PROJECT_STILL_EVENT, ({ payload: file }) => {
        loadThumbnailRef.current(file);
        // The strip is composed after the still; either may be new.
        forgetStripRef.current(file);
      }),
      getCurrentWindow().onFocusChanged(({ payload: focused }) => {
        if (focused) reload();
      }),
    ];
    return () => {
      for (const subscription of subscriptions) {
        void subscription.then((unlisten) => {
          unlisten();
        });
      }
    };
  }, []);

  useEffect(() => {
    for (const project of projects ?? []) {
      const key = `${project.file}@${String(project.modifiedMs)}`;
      if (!project.available || requestedRef.current.has(key)) continue;
      requestedRef.current.add(key);
      loadThumbnailRef.current(project.file, key);
    }
  }, [projects]);

  const select = (next: string) => {
    // Pressing the row already chosen asks for nothing new; clearing the list
    // then would leave it empty, with no change of selection to refill it.
    if (next === selected) return;
    setProjects(null);
    setSelected(next);
  };
  const run = (action: Promise<unknown>) => {
    setError(null);
    action
      .then(() => {
        setGeneration((value) => value + 1);
      })
      .catch((cause: unknown) => {
        setError(message(cause));
      });
  };
  const undo = useUndoDelete();
  const copy = useProjectCopy(run);
  const scrub = useScrubStrips();
  forgetStripRef.current = scrub.forget;
  const titleOf = (file: string | undefined) =>
    projects?.find((project) => project.file === file)?.title ?? "";

  return (
    <ProjectBrowser
      copying={copy.copying}
      deleted={undo.deleted}
      error={error}
      locations={locations}
      onAddLocation={() => {
        run(
          addProjectLocation().then((added) => {
            if (added) select(added);
          }),
        );
      }}
      onClose={() => void hideProjectBrowser()}
      onDelete={(files) => {
        const title = titleOf(files[0]);
        run(
          Promise.all(files.map(deleteProject)).then((deleted) => {
            undo.record(deleted, title);
          }),
        );
      }}
      // One copy at a time: the notice follows one.
      onDuplicate={
        copy.copying
          ? undefined
          : (files) => {
              copy.start(
                {
                  count: files.length,
                  destination: null,
                  kind: "duplicate",
                  title: titleOf(files[0]),
                },
                duplicateProjects(files),
              );
            }
      }
      onEmpty={() => {
        run(emptyRecentlyDeleted());
      }}
      onForget={(file) => {
        run(forgetRecentProject(file));
      }}
      onMinimize={() => void getCurrentWindow().minimize()}
      onMove={
        copy.copying
          ? undefined
          : (files, location) => {
              copy.start(
                {
                  count: files.length,
                  destination:
                    locations.find(({ path }) => path === location)?.name ??
                    null,
                  kind: "move",
                  title: titleOf(files[0]),
                },
                moveProjects(files, location),
              );
            }
      }
      onOpen={(file) => {
        run(openProjectFile(file));
      }}
      onOpenFile={() => {
        run(openOtherProject());
      }}
      onOpenLocation={(path) => {
        run(openProjectLocation(path));
      }}
      onRemoveLocation={(path) => {
        if (path === selected) select(RECENT);
        run(removeProjectLocation(path));
      }}
      onRename={(file, title) => {
        setError(null);
        return renameProject(file, title).then(
          (renamed) => {
            // The card is listed again under its new manifest; the still it
            // showed stays with it, so it is not drawn empty until the new
            // name's still has been asked for.
            setThumbnails((current) => ({
              ...current,
              [renamed]: current[file] ?? null,
            }));
            setGeneration((value) => value + 1);
          },
          (cause: unknown) => {
            setError(message(cause));
            throw cause;
          },
        );
      }}
      onRequestScrubStrip={scrub.request}
      onRestore={(files) => {
        run(restoreProjects(files));
      }}
      onReveal={(file) => {
        run(revealProject(file));
      }}
      onSelect={select}
      onTrash={(files) => {
        run(trashProjects(files));
      }}
      onUndoDelete={() => {
        run(restoreProjects(undo.take()));
      }}
      projects={projects}
      scrubStrips={scrub.strips}
      selected={selected}
      thumbnails={thumbnails}
    />
  );
}
