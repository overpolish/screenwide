// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Flag, Mic, type LucideIcon } from "lucide-react";
import { motion, useReducedMotion } from "motion/react";
import { type ReactNode, useState } from "react";

import { t } from "../../../i18n/i18n";
import { motionDurations, motionEasings } from "../../../lib/motion";
import { ListBox } from "../../../primitives/listbox/listbox";
import { ListBoxItem } from "../../../primitives/listbox-item/listbox-item";
import { Text } from "../../../primitives/text/text";
import { formatDuration } from "../../editor/duration";
import { BEAT_GLYPHS } from "../../editor/glyphs/beat-glyphs";
import { foundText } from "../../editor/tools/transcript/transcript-found-text";

import type { Find, FindKind, SearchResult } from "./project-search";
import type { TimeMap } from "../../editor/timeline/timeline-scale";

// What a search found, as moments: each project it was found in, its card
// already at the first find, and beside it every find, with what matched
// marked. Pointing at a find turns the card to it; choosing one opens the
// project, at its start, as the editor takes no start position.

const GLYPHS: Record<FindKind, LucideIcon> = {
  moment: Flag,
  note: Mic,
  said: BEAT_GLYPHS.speech,
};

function Marked({ find }: { find: Find }) {
  return (
    <span className="min-w-0 truncate" dir="auto">
      {foundText(find.text, [find.match])}
    </span>
  );
}

export type SearchResultRowProps = {
  /** Draws the project's card, turned to a find while one is pointed at. */
  card: (shown: { hits: number[]; previewAt: number | null }) => ReactNode;
  index: number;
  /** How the recording's time plays now, for the finds' times; none while
   * it is read. */
  map: TimeMap | null;
  /** Opens the project; absent where it cannot be opened. */
  onOpen: (() => void) | undefined;
  result: SearchResult;
};

export function SearchResultRow({
  card,
  index,
  map,
  onOpen,
  result,
}: SearchResultRowProps) {
  const reducedMotion = useReducedMotion() ?? false;
  const [aimed, setAimed] = useState<number | null>(null);
  const { finds, project } = result;
  return (
    <motion.section
      animate={{ opacity: 1, y: 0 }}
      aria-label={project.title}
      className="grid grid-cols-[minmax(13rem,20rem)_minmax(0,1fr)] items-start gap-group"
      initial={reducedMotion ? { opacity: 0 } : { opacity: 0, y: 8 }}
      transition={{
        delay: Math.min(index, 8) * 0.04,
        duration: motionDurations.travel,
        ease: motionEasings.out,
      }}
    >
      {card({
        hits: finds.map(({ at }) => at),
        previewAt: aimed === null ? null : (finds[aimed]?.at ?? null),
      })}
      {/* Kept to a reading width, so on a wide window a find's time stays
          beside its line rather than across the pane. */}
      <div className="flex max-w-xl min-w-0 flex-col gap-inline">
        <Text className="ps-inset-sm text-fg-secondary" variant="caption">
          {t("project-browser-search-finds", { count: finds.length })}
        </Text>
        {finds.length > 0 ? (
          <ListBox
            aria-label={t("project-browser-search-finds-in", {
              title: project.title,
            })}
            className="w-full p-0"
            onAction={onOpen}
          >
            {finds.map((find, findIndex) => {
              const Glyph = GLYPHS[find.kind];
              return (
                <ListBoxItem
                  id={String(findIndex)}
                  key={`${String(find.at)}-${find.kind}`}
                  onHoverChange={(isHovered) => {
                    setAimed(isHovered ? findIndex : null);
                  }}
                  showsSelection={false}
                  textValue={find.text}
                >
                  <span className="flex min-w-0 items-center gap-inline">
                    <Glyph
                      aria-hidden
                      className="size-icon shrink-0 text-fg-secondary"
                    />
                    <Marked find={find} />
                    <span className="ms-auto shrink-0 text-caption text-fg-tertiary tabular-nums">
                      {formatDuration(
                        (map ? map.toPlayed(find.at) : find.at) * 1000,
                      )}
                    </span>
                  </span>
                </ListBoxItem>
              );
            })}
          </ListBox>
        ) : null}
      </div>
    </motion.section>
  );
}
