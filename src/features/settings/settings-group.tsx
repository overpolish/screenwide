// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ReactNode } from "react";

import { Text } from "../../primitives/text/text";

import { SettingsReveal } from "./settings-reveal";

// A group of a settings pane's rows, in a box of the fill that a notice
// sits in, so each group reads as one thing and its heading as its name
// rather than one more row. The heading and any note sit above the box,
// flush with its edge, as a label for the whole box. A pane lays groups `gap-group`
// apart, and leaves its first group untitled when the title would repeat
// the pane's own. A note that comes and goes with a setting, such as one
// saying a tool is off, opens and closes under the heading.
export function SettingsGroup({
  children,
  description,
  footnote,
  title,
}: {
  children: ReactNode;
  /** A note on the whole group, under its title. */
  description?: ReactNode;
  /** A note after the box, such as how to use what it sets. */
  footnote?: ReactNode;
  title?: ReactNode;
}) {
  return (
    <section className="flex flex-col gap-inline">
      {title || description ? (
        <div className="flex flex-col">
          {title ? (
            <Text as="h2" variant="heading">
              {title}
            </Text>
          ) : null}
          <SettingsReveal>
            {description ? <Text variant="caption">{description}</Text> : null}
          </SettingsReveal>
        </div>
      ) : null}
      <div className="flex flex-col gap-separate rounded-panel bg-fill p-inset">
        {children}
      </div>
      {footnote ? <Text variant="caption">{footnote}</Text> : null}
    </section>
  );
}
