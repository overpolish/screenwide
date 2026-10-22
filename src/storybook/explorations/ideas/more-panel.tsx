// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { motion } from "motion/react";
import { useState } from "react";

import { ControlRow } from "../../../composites/control-row/control-row";
import { Button } from "../../../primitives/button/button";
import { PillGroup } from "../../../primitives/pill-group/pill-group";
import { Switch } from "../../../primitives/switch/switch";
import { Text } from "../../../primitives/text/text";

import { type Pen } from "./looks";

// Everything a mark has beyond its colour, size and look: when it comes and
// goes and how it moves. Rare choices, so they wait here rather than greet
// every new mark.

const TITLES: Record<Pen, string> = {
  arrow: "Arrow",
  counter: "Counter",
  focus: "Focus",
};

export function MorePanel({ onClose, pen }: { onClose: () => void; pen: Pen }) {
  const [appears, setAppears] = useState("click");
  const [animation, setAnimation] = useState(
    pen === "counter" ? "pop" : "draw",
  );
  const [stays, setStays] = useState("step");
  const [hasShadow, setHasShadow] = useState(true);
  return (
    <motion.aside
      animate={{ opacity: 1, x: 0 }}
      className="flex w-80 flex-col gap-separate rounded-panel bg-fill p-inset"
      exit={{ opacity: 0, x: 12 }}
      initial={{ opacity: 0, x: 12 }}
      transition={{ bounce: 0, duration: 0.25, type: "spring" }}
    >
      <Text as="h2" variant="heading">
        {TITLES[pen]}
      </Text>
      <ControlRow title="Appears">
        {(controlProps) => (
          <PillGroup
            {...controlProps}
            aria-label="Appears"
            items={[
              { id: "click", label: "At its click" },
              { id: "playhead", label: "Here" },
            ]}
            onSelectionChange={setAppears}
            selected={appears}
          />
        )}
      </ControlRow>
      <ControlRow title="Comes in">
        {(controlProps) => (
          <PillGroup
            {...controlProps}
            aria-label="Comes in"
            items={[
              { id: "draw", label: "Draws on" },
              { id: "pop", label: "Pops" },
              { id: "fade", label: "Fades" },
            ]}
            onSelectionChange={setAnimation}
            selected={animation}
          />
        )}
      </ControlRow>
      <ControlRow title="Stays">
        {(controlProps) => (
          <PillGroup
            {...controlProps}
            aria-label="Stays"
            items={[
              { id: "step", label: "This step" },
              { id: "short", label: "3 s" },
            ]}
            onSelectionChange={setStays}
            selected={stays}
          />
        )}
      </ControlRow>
      <ControlRow title="Shadow">
        {(controlProps) => (
          <Switch
            {...controlProps}
            isSelected={hasShadow}
            onChange={setHasShadow}
          />
        )}
      </ControlRow>
      <div className="flex justify-end">
        <Button onPress={onClose}>Done</Button>
      </div>
    </motion.aside>
  );
}
