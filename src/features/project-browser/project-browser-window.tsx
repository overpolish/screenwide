// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { convertFileSrc } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect, useRef, useState } from "react";

import {
  addProjectLocation,
  forgetRecentProject,
  getProjectLocations,
  getProjectThumbnail,
  hideProjectBrowser,
  listProjects,
  openOtherProject,
  openProjectFile,
  openProjectLocation,
  PROJECT_STILL_EVENT,
  PROJECTS_CHANGED_EVENT,
  removeProjectLocation,
  renameProject,
  revealProject,
  trashProject,
} from "./api";
import { ProjectBrowser } from "./project-browser";
import {
  RECENT,
  sourceFor,
  type ProjectLocation,
  type ProjectSummary,
} from "./types";

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

  useEffect(() => {
    const reload = () => {
      setGeneration((value) => value + 1);
    };
    const subscriptions = [
      listen(PROJECTS_CHANGED_EVENT, reload),
      listen<string>(PROJECT_STILL_EVENT, ({ payload: file }) => {
        loadThumbnailRef.current(file);
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

  return (
    <ProjectBrowser
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
      onForget={(file) => {
        run(forgetRecentProject(file));
      }}
      onMinimize={() => void getCurrentWindow().minimize()}
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
      onReveal={(file) => {
        run(revealProject(file));
      }}
      onSelect={select}
      onTrash={(file) => {
        run(trashProject(file));
      }}
      projects={projects}
      selected={selected}
      thumbnails={thumbnails}
    />
  );
}
