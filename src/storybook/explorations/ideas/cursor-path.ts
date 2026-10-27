// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ACTIONS, type Action, centreOf, elementById } from "./ideas-data";

// Where the recording's own pointer was: it travels between the elements it
// clicked, slowing into each the way a hand does. The ideas share one
// recording; the editor passes the actions of the one it edits.

type Stop = { at: number; x: number; y: number };

const stopsByActions = new WeakMap<Action[], Stop[]>();

function stopsOf(actions: Action[]) {
  const known = stopsByActions.get(actions);
  if (known) return known;
  const stops = [
    { at: 0, x: 60, y: 50 },
    ...actions.flatMap(({ at, element }) => {
      const target = elementById(element);
      return target ? [{ at, ...centreOf(target) }] : [];
    }),
  ];
  stopsByActions.set(actions, stops);
  return stops;
}

const minJerk = (u: number) => u * u * u * (10 - 15 * u + 6 * u * u);

export function cursorAt(seconds: number, actions = ACTIONS) {
  const stops = stopsOf(actions);
  const next = stops.findIndex(({ at }) => at > seconds);
  if (next <= 0) {
    const last = next === 0 ? stops[0] : stops[stops.length - 1];
    return { x: last.x, y: last.y };
  }
  const from = stops[next - 1];
  const to = stops[next];
  // Rest, then move over the last 1.2 s before the click.
  const travel = Math.min(1.2, to.at - from.at);
  const u = Math.min(Math.max((seconds - (to.at - travel)) / travel, 0), 1);
  const eased = minJerk(u);
  return {
    x: from.x + (to.x - from.x) * eased,
    y: from.y + (to.y - from.y) * eased,
  };
}

/** How recently there was a click, 0 to 1, for its ripple. */
export function clickAt(seconds: number, actions = ACTIONS) {
  const click = actions.find(
    ({ at, kind }) => kind === "click" && seconds >= at && seconds - at < 0.5,
  );
  return click ? (seconds - click.at) / 0.5 : null;
}
