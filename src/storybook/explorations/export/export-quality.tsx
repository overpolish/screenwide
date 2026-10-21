// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";
import { RadioButton, RadioField, RadioGroup } from "react-aria-components";

import { ControlRow } from "../../../composites/control-row/control-row";
import { cn, elementFocusVisible, focusStyles } from "../../../lib/styling";
import { Button } from "../../../primitives/button/button";
import { ChangedDot } from "../shared/changed-dot";

import { BALANCED, type QualityStep } from "./export-data";
import { Choose } from "./export-parts";

// The quality ladder: four presets as cards, each saying what it makes, its
// size and the file it comes out at, so the choice is by result rather than
// by setting. Drawn as a segmented control is: a track on the fill, the
// chosen card in the primary colour, one of a set read at a glance.
// Customise sets size and compression apart, as today's two settings, and
// still estimates the file.

const CARD_RADIUS = "rounded-[calc(var(--radius-control)-0.125rem)]";

function Presets({ steps }: { steps: QualityStep[] }) {
  const [chosen, setChosen] = useState(steps[BALANCED]?.label ?? "");
  return (
    <RadioGroup
      aria-label="Quality"
      className="grid grid-cols-4 gap-0.5 rounded-control bg-fill p-0.5"
      onChange={setChosen}
      orientation="horizontal"
      value={chosen}
    >
      {steps.map(({ dimensions, label, size }) => (
        <RadioField key={label} value={label}>
          <RadioButton
            className={cn(
              "group flex w-full cursor-default flex-col items-center py-2 text-fg-secondary transition-colors",
              CARD_RADIUS,
              "data-[hovered]:not-data-[selected]:text-fg",
              "data-[selected]:bg-primary data-[selected]:text-on-primary data-[selected]:primary-raised",
              focusStyles,
              elementFocusVisible,
            )}
          >
            <span className="text-label">{label}</span>
            <span className="text-caption tabular-nums opacity-70">
              {dimensions}
            </span>
            <span className="text-caption tabular-nums">{size}</span>
          </RadioButton>
        </RadioField>
      ))}
    </RadioGroup>
  );
}

function Custom() {
  return (
    <div className="flex flex-col gap-inline">
      <ControlRow title="Size" titleAccessory={<ChangedDot setting="Size" />}>
        {() => (
          <Choose
            className="w-36"
            items={[{ id: "2880", label: "2880 × 1620" }]}
            label="Size"
          />
        )}
      </ControlRow>
      <ControlRow title="Compression">
        {() => (
          <Choose
            className="w-36"
            items={[{ id: "balanced", label: "Balanced" }]}
            label="Compression"
          />
        )}
      </ControlRow>
      <span className="text-caption text-fg-tertiary tabular-nums">
        About 58 MB
      </span>
    </div>
  );
}

export function Quality({
  initialCustomised,
  steps,
}: {
  initialCustomised: boolean;
  steps: QualityStep[];
}) {
  const [isCustomised, setIsCustomised] = useState(initialCustomised);
  return (
    <section className="flex flex-col gap-inline">
      <span className="flex items-center justify-between">
        <span className="text-heading">Quality</span>
        <Button
          onPress={() => {
            setIsCustomised(!isCustomised);
          }}
          variant="ghost"
        >
          {isCustomised ? "Use a preset" : "Customise"}
        </Button>
      </span>
      {isCustomised ? <Custom /> : <Presets steps={steps} />}
    </section>
  );
}
