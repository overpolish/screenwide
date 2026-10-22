// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AnimatePresence, motion } from "motion/react";
import { useState } from "react";

import { cn } from "../../../lib/styling";
import { Button } from "../../../primitives/button/button";

import {
  describe,
  ICONS,
  type Line,
  LINES,
  STEPS,
} from "./action-script-lines";
import { IdeaFrame } from "./idea-frame";
import { boxStyle, clock, elementById, RECORDING_S } from "./ideas-data";
import { MockApp } from "./mock-app";
import { RecordedCursor } from "./recorded-cursor";

// The recording as a script: what was said, and what was done between, in
// the order it happened. Each line owns the time up to the next one, so
// deleting a line deletes its time. Steps break where the work moved on.

export function ActionScript() {
  const [removed, setRemoved] = useState<Set<string>>(() => new Set());
  const [selected, setSelected] = useState<Set<string>>(() => new Set());
  const [anchor, setAnchor] = useState<string | null>(null);
  const [hovered, setHovered] = useState<Line | null>(null);
  const kept = LINES.filter(({ id }) => !removed.has(id));
  const total = kept.reduce((sum, { at, end }) => sum + end - at, 0);
  const shown = hovered ?? LINES[0];
  const target = elementById(shown.action?.element);

  const remove = (ids: Iterable<string>) => {
    setRemoved((current) => new Set([...current, ...ids]));
    setSelected(new Set());
  };
  const restore = (id: string) => {
    setRemoved((current) => {
      const next = new Set(current);
      next.delete(id);
      return next;
    });
  };

  return (
    <IdeaFrame
      hint="Hover a line to see that moment. Click to select, Shift-click for a range, then press Delete. A removed line leaves a seam you can restore."
      pitch="The recording reads as a script of what was said and what was done. Delete a line and its time goes with it; idle stretches are already flagged; steps become chapters."
      title="Action script"
    >
      <div className="flex min-h-0 w-full max-w-[1000px] grow gap-group">
        <div
          aria-label="Script"
          className="flex min-h-0 w-[440px] shrink-0 flex-col overflow-y-auto rounded-panel bg-fill p-inset-sm outline-none"
          onKeyDown={(event) => {
            if (
              (event.key === "Delete" || event.key === "Backspace") &&
              selected.size > 0
            ) {
              event.preventDefault();
              remove(selected);
            }
          }}
          onPointerLeave={() => {
            setHovered(null);
          }}
          role="listbox"
          tabIndex={0}
        >
          {LINES.map((line, index) => {
            const step = STEPS.find(
              ({ at }) => at <= line.at && (LINES[index - 1]?.at ?? -1) < at,
            );
            const isRemoved = removed.has(line.id);
            const isSelected = selected.has(line.id);
            const action = line.action;
            const Icon = action ? ICONS[action.kind] : null;
            return (
              <div key={line.id}>
                {step ? (
                  <div className="px-inset-sm pt-separate pb-inline text-label font-semibold text-fg-secondary">
                    {step.title}
                  </div>
                ) : null}
                <AnimatePresence initial={false} mode="popLayout">
                  {isRemoved ? (
                    <motion.div
                      animate={{ opacity: 1 }}
                      className="flex h-6 items-center gap-inline px-inset-sm"
                      exit={{ opacity: 0 }}
                      initial={{ opacity: 0 }}
                      key="seam"
                      transition={{ duration: 0.15 }}
                    >
                      <span className="h-px grow bg-fg-tertiary/40" />
                      <span className="text-caption text-fg-tertiary tabular-nums">
                        {(line.end - line.at).toFixed(1)} s removed
                      </span>
                      <button
                        className="text-caption text-fg-secondary underline-offset-2 hover:underline"
                        onClick={() => {
                          restore(line.id);
                        }}
                        type="button"
                      >
                        Restore
                      </button>
                      <span className="h-px grow bg-fg-tertiary/40" />
                    </motion.div>
                  ) : (
                    <motion.div
                      animate={{ height: "auto", opacity: 1 }}
                      aria-selected={isSelected}
                      className={cn(
                        "grid cursor-default grid-cols-[52px_1fr] items-start gap-inline overflow-hidden rounded-part px-inset-sm py-1",
                        isSelected
                          ? "bg-primary text-on-primary"
                          : "hover:bg-fill",
                      )}
                      exit={{ height: 0, opacity: 0 }}
                      initial={{ height: 0, opacity: 0 }}
                      key="line"
                      onClick={(event) => {
                        if (event.shiftKey && anchor) {
                          const from = LINES.findIndex(
                            ({ id }) => id === anchor,
                          );
                          const [low, high] = [
                            Math.min(from, index),
                            Math.max(from, index),
                          ];
                          setSelected(
                            new Set(
                              LINES.slice(low, high + 1).map(({ id }) => id),
                            ),
                          );
                        } else {
                          setSelected(new Set([line.id]));
                          setAnchor(line.id);
                        }
                      }}
                      onPointerEnter={() => {
                        setHovered(line);
                      }}
                      role="option"
                      transition={{ bounce: 0, duration: 0.3, type: "spring" }}
                    >
                      <span
                        className={cn(
                          "pt-0.5 text-caption tabular-nums",
                          isSelected
                            ? "text-on-primary/70"
                            : "text-fg-tertiary",
                        )}
                      >
                        {clock(line.at)}
                      </span>
                      {action && Icon ? (
                        <span className="flex items-center gap-inline">
                          <span
                            className={cn(
                              "flex items-center gap-1 rounded-sm px-1.5 py-0.5 text-label",
                              isSelected
                                ? "bg-on-primary/15"
                                : "bg-fill-strong",
                              action.kind === "idle" &&
                                !isSelected &&
                                "text-fg-tertiary",
                            )}
                          >
                            <Icon className="size-3.5 shrink-0" />
                            {describe(action)}
                          </span>
                          {action.kind === "idle" ? (
                            <Button
                              onPress={() => {
                                remove([line.id]);
                              }}
                              variant="ghost"
                            >
                              Remove
                            </Button>
                          ) : null}
                        </span>
                      ) : (
                        <span className="text-body">{line.text}</span>
                      )}
                    </motion.div>
                  )}
                </AnimatePresence>
              </div>
            );
          })}
        </div>
        <div className="flex min-w-0 grow flex-col gap-separate">
          <div className="relative overflow-hidden rounded-part shadow-lg">
            <MockApp>
              {target ? (
                <span
                  className="absolute rounded-[10px] shadow-[0_0_0_3px_rgba(10,132,255,0.9)] transition-all duration-200"
                  style={boxStyle(target)}
                />
              ) : null}
              <RecordedCursor seconds={shown.at + 0.05} />
            </MockApp>
          </div>
          <div className="flex items-baseline justify-between">
            <span className="text-heading tabular-nums">
              {clock(RECORDING_S)} <span className="text-fg-tertiary">→</span>{" "}
              {clock(total)}
            </span>
            <span className="text-caption text-fg-tertiary">
              {LINES.length - kept.length} of {LINES.length} lines removed
            </span>
          </div>
          {/* What is left, at its new length. */}
          <div
            className="flex h-6 gap-px overflow-hidden rounded-part"
            dir="ltr"
          >
            {LINES.map((line) => (
              <motion.span
                animate={{
                  flexGrow: removed.has(line.id) ? 0.0001 : line.end - line.at,
                }}
                className={cn(
                  "h-full min-w-0",
                  line.action ? "bg-fill-strong" : "bg-fg-tertiary/50",
                )}
                key={line.id}
                style={{ flexBasis: 0 }}
                transition={{ bounce: 0, duration: 0.35, type: "spring" }}
              />
            ))}
            <motion.span
              animate={{ flexGrow: RECORDING_S - total }}
              className="h-full"
              style={{ flexBasis: 0 }}
              transition={{ bounce: 0, duration: 0.35, type: "spring" }}
            />
          </div>
        </div>
      </div>
    </IdeaFrame>
  );
}
