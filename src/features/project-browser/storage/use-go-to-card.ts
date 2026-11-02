// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useReducedMotion } from "motion/react";
import { type RefObject, useEffect, useRef, useState } from "react";

import { filterByKind, type KindFilter } from "../list/project-list";

import type { ProjectSummary } from "../types";

// Going to a project's card from its segment in the storage strip. The
// strip draws the source's projects, so the card is always in the list
// open; only the kind filter or the search can hide it, and whichever does
// is lifted. The card is then scrolled to the middle of the pane, clear of
// the fades at its ends and of the island at its foot, given focus, and
// outlined for a moment as pointing at its segment outlines it, since a
// focus ring shows only after the keyboard.

/** How long the card stays outlined, long enough to be found by eye after
 * the scroll ends. */
const MARKED_MS = 1_500;

/** A press on a segment; each press is a new one, so pressing the same
 * segment again goes to its card again. */
export type GoToCard = { file: string };

export function useGoToCard({
  kind,
  onClearSearch,
  onKindChange,
  projects,
  request,
  rootRef,
  search,
  shown,
}: {
  /** The kind filter in effect. */
  kind: KindFilter;
  onClearSearch: () => void;
  onKindChange: (kind: KindFilter) => void;
  projects: readonly ProjectSummary[];
  request: GoToCard | null;
  rootRef: RefObject<HTMLElement | null>;
  /** What the search finds among `within`, or null with no search. */
  search: ((within: readonly ProjectSummary[]) => ProjectSummary[]) | null;
  shown: readonly ProjectSummary[];
}) {
  const reducedMotion = useReducedMotion() ?? false;
  // A request already there as the pane mounts belongs to the source it
  // was made in, before the pane was opened on another.
  const [atMount] = useState(request);
  // The request whose outline has had its time.
  const [faded, setFaded] = useState<GoToCard | null>(null);
  // The card still to scroll to, once it is drawn and its pane scrolls.
  const pendingRef = useRef<string | null>(null);
  const viewportRef = useRef<HTMLElement | null>(null);

  const latestRef = useRef({
    kind,
    onClearSearch,
    onKindChange,
    projects,
    search,
    shown,
  });
  useEffect(() => {
    latestRef.current = {
      kind,
      onClearSearch,
      onKindChange,
      projects,
      search,
      shown,
    };
  });

  const scrollToPending = () => {
    const file = pendingRef.current;
    const viewport = viewportRef.current;
    if (file === null || !viewport) return;
    const picture = rootRef.current?.querySelector<HTMLElement>(
      `[data-project-file="${CSS.escape(file)}"] [data-project-picture]`,
    );
    if (!picture || !viewport.contains(picture)) return;
    pendingRef.current = null;
    picture.focus({ preventScroll: true });
    // A card just drawn can make the pane longer than it was, and the
    // `ScrollArea` lets it scroll only from the next frame; a smooth scroll
    // started before then is cut short.
    requestAnimationFrame(() => {
      requestAnimationFrame(() => {
        picture.scrollIntoView({
          behavior: reducedMotion ? "instant" : "smooth",
          block: "center",
        });
      });
    });
  };

  useEffect(() => {
    if (!request || request === atMount) return;
    const { file } = request;
    const latest = latestRef.current;
    const project = latest.projects.find((each) => each.file === file);
    if (!project) return;
    if (!latest.shown.some((each) => each.file === file)) {
      const kindHides = latest.kind !== "all" && project.kind !== latest.kind;
      if (kindHides) latest.onKindChange("all");
      const within = kindHides
        ? latest.projects
        : filterByKind(latest.projects, latest.kind);
      if (latest.search?.(within).every((each) => each.file !== file))
        latest.onClearSearch();
    }
    pendingRef.current = file;
    const timer = window.setTimeout(() => {
      setFaded(request);
      // A card that never came is not scrolled to later out of the blue.
      if (pendingRef.current === file) pendingRef.current = null;
    }, MARKED_MS);
    return () => {
      window.clearTimeout(timer);
    };
  }, [atMount, request]);

  // After each drawing: a card the filters hid is drawn only once they are
  // lifted.
  useEffect(scrollToPending);

  return {
    /** The card outlined for having been gone to. */
    marked:
      request && request !== atMount && request !== faded ? request.file : null,
    /** For the pane's `ScrollArea`, which scrolls only once set up: a card
     * shown by lifting a search that found nothing waits for it. */
    onViewport: (viewport: HTMLElement) => {
      viewportRef.current = viewport;
      scrollToPending();
    },
  };
}
