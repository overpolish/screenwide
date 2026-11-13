// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { lookUp, parseWork } from "./work";

const ID = "4bc43a04-ef46-4544-a0c1-63c63f56e276";

const workOf = (query: string) => {
  const asked = parseWork(new URL(`https://w.test/work?${query}`));
  if (typeof asked === "string") throw new Error(asked);
  return asked;
};

const answering = (response: Response) => {
  const asked: string[] = [];
  return {
    asked,
    fetcher: (input: URL) => {
      asked.push(input.toString());
      return Promise.resolve(response);
    },
  };
};

const tokens = () => Promise.resolve("token");

describe("a work looked up again", () => {
  it("is asked for at its kind's endpoint", async () => {
    const { asked, fetcher } = answering(
      Response.json({ id: ID, license: "cc0", url: "https://cdn.test/a.mp3" }),
    );
    const found = await lookUp(workOf(`kind=effect&id=${ID}`), tokens, fetcher);
    expect(asked).toEqual([`https://api.openverse.org/v1/audio/${ID}/`]);
    expect(found?.license).toBe("cc0");
  });

  it("is withdrawn where its licence is no longer one the app may use, or it is gone", async () => {
    const changed = answering(
      Response.json({
        id: ID,
        license: "by-nc",
        url: "https://cdn.test/a.mp3",
      }),
    );
    expect(
      await lookUp(workOf(`kind=music&id=${ID}`), tokens, changed.fetcher),
    ).toBeNull();
    const gone = answering(new Response(null, { status: 404 }));
    expect(
      await lookUp(workOf(`kind=image&id=${ID}`), tokens, gone.fetcher),
    ).toBeNull();
  });

  it("refuses an id that is not Openverse's", () => {
    for (const query of [`kind=music&id=../../x`, `kind=video&id=${ID}`])
      expect(typeof parseWork(new URL(`https://w.test/work?${query}`))).toBe(
        "string",
      );
  });
});
