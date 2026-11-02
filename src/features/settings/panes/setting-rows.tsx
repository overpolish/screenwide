// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ControlRow } from "../../../composites/control-row/control-row";
import {
  PillGroup,
  type PillGroupItem,
} from "../../../primitives/pill-group/pill-group";
import { Switch } from "../../../primitives/switch/switch";

import type { ToggleItem } from "../sections/general-rows";

// The two kinds of row General and Recording are made of: a switch that
// takes effect at once, and a choice of a few values as pills.

export function ToggleRow({
  isDisabled,
  isSelected,
  item,
  onChange,
}: {
  isDisabled: boolean;
  isSelected: boolean;
  item: ToggleItem;
  onChange: (isSelected: boolean) => void;
}) {
  return (
    <ControlRow description={item.description} title={item.title}>
      {(controlProps) => (
        <Switch
          {...controlProps}
          isDisabled={isDisabled}
          isSelected={isSelected}
          onChange={onChange}
        />
      )}
    </ControlRow>
  );
}

export function ChoiceRow({
  description,
  isDisabled,
  items,
  onChange,
  selected,
  title,
}: {
  isDisabled: boolean;
  items: PillGroupItem[];
  onChange: (id: string) => void;
  selected: string;
  title: string;
  description?: string;
}) {
  return (
    <ControlRow description={description} title={title}>
      {(controlProps) => (
        <div {...controlProps} role="group">
          <PillGroup
            aria-label={title}
            isDisabled={isDisabled}
            items={items}
            onSelectionChange={onChange}
            selected={selected}
          />
        </div>
      )}
    </ControlRow>
  );
}
