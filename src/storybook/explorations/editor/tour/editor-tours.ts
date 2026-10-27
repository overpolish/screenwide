// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { find, startTour } from "./tour";

// How the editor's core flows are done, played through by hand: adding an
// annotation, timing one, setting scenes and squeezing pauses. Each starts
// from the editor as a story opens it and leaves it live when it ends.

const all = (root: ParentNode, selector: string) => [
  ...root.querySelectorAll<HTMLElement>(selector),
];

/** Something on the recorded screen, by the words on it. */
const onScreen = (root: ParentNode, label: string) =>
  all(root, "main span.relative").find((span) => span.textContent === label)
    ?.parentElement ?? null;

/** A button by its words or, for a circle with none, its name, such as a
 * scene on the island. */
const button = (root: ParentNode, label: string) =>
  all(root, "button").find(
    (item) =>
      item.textContent.trim() === label ||
      item.getAttribute("aria-label") === label,
  ) ?? null;

const bar = (root: ParentNode, id: number) =>
  root.querySelector<HTMLElement>(`[aria-label="Annotation ${String(id)}"]`);

/** The selected annotation's bar on the timeline. */
const selectedBar = (root: ParentNode) =>
  all(root, '[aria-label^="Annotation "]').find((item) =>
    item.querySelector(".bg-primary"),
  ) ?? null;

/** A selected bar's ends: its start, then its end. */
const ends = (item: HTMLElement | null) =>
  item ? all(item, ".cursor-ew-resize") : [];

/** Where a beat begins on the timeline, level with an annotation's bar. */
const beatStart = (root: ParentNode, title: string, level: HTMLElement) => {
  const beat = root
    .querySelector(`[title="${title}"]`)
    ?.getBoundingClientRect();
  const row = level.getBoundingClientRect();
  return { x: beat?.left ?? row.right, y: row.top + row.height / 2 };
};

/** A scene on the rail along the top of the timeline. */
const sceneClip = (root: ParentNode, index: number) =>
  all(root, '[aria-label^="Scene, "]')[index] as HTMLElement | undefined;

const required = <T>(value: T | null | undefined, what: string): T => {
  if (value === null || value === undefined)
    throw new Error(`The tour could not find ${what}`);
  return value;
};

export async function addingAnAnnotation(root: HTMLElement) {
  const counter = await find(() =>
    root.querySelector('[aria-label="Counter"]'),
  );
  const tour = startTour(root);
  await tour.wait(500);
  await tour.say("Pick a pen: its island sets how it draws");
  await tour.click(counter);
  await tour.say(
    "Point at what it is for: what is under the pointer is outlined",
    600,
  );
  await tour.moveTo(() => required(onScreen(root, "Blank"), "Blank"), 900);
  await tour.wait(700);
  await tour.moveTo(() => required(onScreen(root, "Create"), "Create"), 800);
  await tour.wait(500);
  await tour.say("Click to place it", 700);
  await tour.click(() => required(onScreen(root, "Create"), "Create"), 200);
  await tour.say("It shows from when Create was clicked", 2_800);
  await tour.say("On the timeline it is a bar across the time it shows", 500);
  await tour.moveTo(() => required(selectedBar(root), "the new bar"), 900);
  await tour.wait(1_800);
  await tour.say("Your turn: pick a pen and point", 200);
  await tour.end();
}

export async function timingAnAnnotation(root: HTMLElement) {
  const arrow = await find(() => bar(root, 3));
  const tour = startTour(root);
  await tour.wait(500);
  await tour.say(
    "An annotation is timed by what happened: this arrow shows from when Share was clicked",
    2_400,
  );
  await tour.say("Drag an end to change when it comes or goes", 600);
  await tour.moveTo(() => required(ends(arrow)[0], "the arrow's start"), 900);
  await tour.drag(
    () =>
      required(root.querySelector('[title="Switched to Safari"]'), "Safari"),
    1_300,
  );
  await tour.say(
    "Ends snap to what happened: here, the switch to Safari",
    2_200,
  );
  await tour.say("Pick another annotation to retime it", 600);
  await tour.click(() => required(bar(root, 2), "the counter"), 900);
  await tour.say("Drag its end on, until Starter is picked", 600);
  const counter = required(bar(root, 2), "the counter");
  await tour.moveTo(() => required(ends(counter)[1], "the counter's end"), 900);
  await tour.drag(() => beatStart(root, "Clicked “Starter”", counter), 1_300);
  await tour.say(
    "It snaps to where that click begins, so it goes as it does",
    2_400,
  );
  await tour.end();
}

export async function settingScenes(root: HTMLElement) {
  const first = await find(() => sceneClip(root, 0));
  const tour = startTour(root);
  await tour.wait(500);
  await tour.say(
    "Scenes run along the top: auto zoom has zoomed in where the work is",
    2_000,
  );
  await tour.say("Press one to change it", 600);
  await tour.click(first, 900);
  await tour.say("Show it side by side with the camera instead", 600);
  await tour.click(() =>
    required(button(root, "Side by side"), "Side by side"),
  );
  await tour.wait(1_400);
  await tour.say("Drag its end to keep it up for longer", 600);
  await tour.moveTo(
    () => required(ends(sceneClip(root, 0) ?? null)[1], "the scene's end"),
    800,
  );
  const clip = required(sceneClip(root, 0), "the scene");
  const box = clip.getBoundingClientRect();
  await tour.drag(
    () => ({ x: box.right + 60, y: box.top + box.height / 2 }),
    1_000,
  );
  await tour.wait(800);
  await tour.say("Or add one at the playhead, from the toolbar", 600);
  await tour.click(() =>
    required(
      root.querySelector('[title="Clicked “New Project”"]'),
      "New Project",
    ),
  );
  await tour.click(() =>
    required(root.querySelector('[aria-label="Scene"]'), "Scene"),
  );
  // The island folds away from the strip and comes out by the toolbar.
  await tour.wait(1_500);
  await tour.click(() =>
    required(button(root, "Add Screen above camera"), "Screen above camera"),
  );
  await tour.wait(1_200);
  await tour.say("Play, and the picture changes as each scene comes", 600);
  await tour.click(() =>
    required(root.querySelector('[aria-label="Play"]'), "Play"),
  );
  await tour.wait(5_200);
  await tour.click(
    () => required(root.querySelector('[aria-label="Pause"]'), "Pause"),
    400,
  );
  await tour.say("Your turn: drag along the empty rail to add a zoom", 200);
  await tour.end();
}

/** A pause on the strip, by how long it was recorded. */
const pauseOf = (root: ParentNode, seconds: string) =>
  root.querySelector<HTMLElement>(`[aria-label="Pause, ${seconds} s"]`);

export async function squeezingPauses(root: HTMLElement) {
  const deadAir = await find(() => pauseOf(root, "8.0"));
  const tour = startTour(root);
  await tour.wait(500);
  await tour.say("Each spring is a pause: time where nothing is done", 600);
  await tour.moveTo(deadAir, 900);
  await tour.wait(900);
  await tour.say("Double-click one to keep it as it was recorded", 600);
  await tour.doubleClick(() => required(pauseOf(root, "8.0"), "the pause"));
  await tour.wait(900);
  await tour.say(
    "Squeeze the strip: the rest give way, the kept one holds",
    600,
  );
  const squeeze = required(
    root.querySelector<HTMLElement>('[aria-label="Squeeze"]'),
    "the strip's end",
  );
  await tour.moveTo(squeeze, 900);
  const from = squeeze.getBoundingClientRect();
  await tour.drag(
    () => ({ x: from.left - 220, y: from.top + from.height / 2 }),
    1_400,
  );
  await tour.wait(900);
  await tour.say("Choose a pause, and drag its end to speed it up", 600);
  await tour.click(() => required(pauseOf(root, "5.9"), "the pause"));
  const handle = () =>
    required(
      pauseOf(root, "5.9")?.querySelector<HTMLElement>("[data-pause-end]"),
      "its end",
    );
  await tour.moveTo(handle, 700);
  const pause = required(
    pauseOf(root, "5.9"),
    "the pause",
  ).getBoundingClientRect();
  await tour.drag(
    () => ({
      x: pause.left + pause.width * 0.4,
      y: pause.top + pause.height / 2,
    }),
    1_000,
  );
  await tour.say(
    "Its badge says how much faster than recorded it now plays",
    2_000,
  );
  await tour.say("Or cut one altogether", 600);
  await tour.click(() => required(pauseOf(root, "7.5"), "the first pause"));
  await tour.click(() =>
    required(root.querySelector('[aria-label="Cut the pause"]'), "Cut"),
  );
  await tour.say("Press again to cut it: nothing goes on one press", 600);
  await tour.click(() =>
    required(root.querySelector('[aria-label="Confirm Cut"]'), "Confirm"),
  );
  await tour.say("Your turn: press any pause", 200);
  await tour.end();
}
