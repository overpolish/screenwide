// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ListOrdered, MessageSquareQuote } from "lucide-react";
import { motion } from "motion/react";
import { useState } from "react";

import { motionDurations, motionEasings } from "../../../lib/motion";
import { cn } from "../../../lib/styling";
import { Text } from "../../../primitives/text/text";
import { ICONS } from "../ideas/action-script-lines";

import { type Hit, type Result } from "./moment-search";
import { ProjectCard } from "./project-card";
import { mmss, type Project } from "./projects-model";

// What a search found, as moments: each project it was found in, its card
// already at the first find, and beside it every moment, with what matched
// marked. Pointing at a moment turns the card to it; pressing one peeks
// there, ready to play.

const travel = { duration: motionDurations.travel, ease: motionEasings.out };

const ICON = { ...ICONS, said: MessageSquareQuote, step: ListOrdered };

function Marked({ hit }: { hit: Hit }) {
  const [from, to] = hit.match;
  return (
    <span className="min-w-0 truncate" dir="auto">
      {hit.text.slice(0, from)}
      <mark className="rounded-[3px] bg-emphasis/30 text-fg">
        {hit.text.slice(from, to)}
      </mark>
      {hit.text.slice(to)}
    </span>
  );
}

function ResultRow({
  index,
  onOpen,
  onPeek,
  peekedId,
  result,
}: {
  index: number;
  onOpen: (project: Project, at: number) => void;
  onPeek: (project: Project, at: number) => void;
  peekedId: string | null;
  result: Result;
}) {
  const [aimed, setAimed] = useState<number | null>(null);
  const { hits, project } = result;
  return (
    <motion.div
      animate={{ opacity: 1, y: 0 }}
      className="grid grid-cols-[15rem_1fr] items-start gap-group"
      initial={{ opacity: 0, y: 8 }}
      layout="position"
      transition={{ ...travel, delay: index * 0.04 }}
    >
      <ProjectCard
        hits={hits.map(({ at }) => at)}
        isPeeked={peekedId === project.id}
        onOpen={(at) => {
          onOpen(project, at);
        }}
        onPeek={(at) => {
          onPeek(project, at);
        }}
        previewAt={aimed === null ? null : (hits[aimed]?.at ?? null)}
        project={project}
      />
      <div className="flex min-w-0 flex-col">
        <Text className="pb-inline text-fg-secondary" variant="caption">
          {hits.length === 0
            ? "Found by its name"
            : `${String(hits.length)} ${hits.length === 1 ? "moment" : "moments"}`}
        </Text>
        {hits.map((hit, hitIndex) => {
          const Icon = ICON[hit.kind];
          return (
            <button
              className={cn(
                "flex h-control-height min-w-0 items-center gap-inline rounded-control px-inset-sm text-start text-body text-fg transition-colors",
                aimed === hitIndex ? "bg-fill-strong" : "hover:bg-fill",
              )}
              key={`${String(hit.at)}-${hit.kind}`}
              onClick={() => {
                onPeek(project, hit.at);
              }}
              onPointerEnter={() => {
                setAimed(hitIndex);
              }}
              onPointerLeave={() => {
                setAimed(null);
              }}
              type="button"
            >
              <Icon className="size-icon shrink-0 text-fg-secondary" />
              <Marked hit={hit} />
              <span className="ms-auto shrink-0 text-caption text-fg-tertiary tabular-nums">
                {mmss(hit.at)}
              </span>
            </button>
          );
        })}
      </div>
    </motion.div>
  );
}

export function SearchResults({
  onOpen,
  onPeek,
  peekedId,
  query,
  results,
}: {
  onOpen: (project: Project, at: number) => void;
  onPeek: (project: Project, at: number) => void;
  peekedId: string | null;
  query: string;
  results: Result[];
}) {
  const moments = results.reduce((sum, { hits }) => sum + hits.length, 0);
  if (results.length === 0)
    return (
      <div className="flex grow flex-col items-center justify-center gap-1 py-16 text-center">
        <Text variant="heading">Nothing said or done matches</Text>
        <Text className="text-fg-secondary" variant="body">
          {`No project mentions “${query.trim()}”.`}
        </Text>
      </div>
    );
  return (
    <div className="flex flex-col gap-group">
      <Text className="ps-0.5" variant="heading">
        {`${String(moments)} moments in ${String(results.length)} projects`}
      </Text>
      {results.map((result, index) => (
        <ResultRow
          index={index}
          key={result.project.id}
          onOpen={onOpen}
          onPeek={onPeek}
          peekedId={peekedId}
          result={result}
        />
      ))}
    </div>
  );
}
