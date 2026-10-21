// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { PillGroup } from "../../primitives/pill-group/pill-group";

import { type AnnotationChoiceDefinition } from "./annotation-choices";

// A way to dress an annotation, offered as a segmented choice: the
// definition says what it is called, its segments, and how the stored value
// maps to them.
export function AnnotationChoice<Value>({
  choice,
  isDisabled,
  onChange,
  value,
}: {
  choice: AnnotationChoiceDefinition<Value>;
  onChange: (value: Value) => void;
  value: Value;
  isDisabled?: boolean;
}) {
  return (
    <PillGroup
      aria-label={choice.label()}
      isDisabled={isDisabled}
      items={choice.items()}
      onSelectionChange={(id) => {
        onChange(choice.pick(id));
      }}
      selected={choice.select(value)}
    />
  );
}
