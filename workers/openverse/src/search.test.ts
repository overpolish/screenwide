// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { foundOf, parseSearch, search, upstreamUrl } from "./search";

const searchOf = (query: string) => {
  const asked = parseSearch(new URL(`https://w.test/search?${query}`));
  if (typeof asked === "string") throw new Error(asked);
  return asked;
};

const result = (license: string) => ({
  attribution: '"Rain" by Ana is licensed under CC BY 4.0.',
  creator: "Ana",
  duration: 4200,
  filetype: "mp3",
  foreign_landing_url: "https://freesound.org/s/1",
  id: "a1",
  license,
  license_url: "https://creativecommons.org/licenses/by/4.0/",
  license_version: "4.0",
  title: "Rain",
  url: "https://cdn.test/rain.mp3",
});

describe("a search", () => {
  it("asks only for what may be used with a credit, of the kind shown", () => {
    const url = upstreamUrl(searchOf("kind=effect&q=Rain%20%20drops&page=2"));
    expect(url.pathname).toBe("/v1/audio/");
    expect(url.searchParams.get("license")).toBe("cc0,pdm,by");
    expect(url.searchParams.get("length")).toBe("shortest");
    expect(url.searchParams.get("q")).toBe("rain drops");
    expect(url.searchParams.get("page")).toBe("2");
    expect(upstreamUrl(searchOf("kind=image&q=arrow")).pathname).toBe(
      "/v1/images/",
    );
  });

  it("asks for songs where they are published, narrowed by their filters", () => {
    const url = upstreamUrl(
      searchOf("kind=music&q=Piano&tempo=slow&vocals=instrumental&mood=calm"),
    );
    expect(url.searchParams.get("excluded_source")).toBe("freesound");
    expect(url.searchParams.get("extension")).toContain("mp32");
    expect(url.searchParams.get("q")).toBe(
      "piano speed_low instrumental peaceful",
    );
    expect(
      upstreamUrl(searchOf("kind=music&mood=happy")).searchParams.get("q"),
    ).toBe("happy");
    expect(
      upstreamUrl(searchOf("kind=effect&q=door&mood=happy")).searchParams.get(
        "q",
      ),
    ).toBe("door");
  });

  it("refuses a search with nothing to find, an unknown kind or filter, or a page past the last", () => {
    for (const query of [
      "kind=music&q=%20",
      "kind=effect&mood=happy",
      "kind=music&mood=angry",
      "kind=video&q=rain",
      "kind=music&q=rain&page=13",
      "kind=music&q=rain&page=0",
    ])
      expect(
        typeof parseSearch(new URL(`https://w.test/search?${query}`)),
      ).toBe("string");
  });

  it("names Jamendo's MP3s as MP3s and describes a song by its tags", () => {
    const found = foundOf({
      ...result("by"),
      filetype: "mp32",
      genres: ["ambient", 3],
      tags: [
        { name: "speed_low" },
        { name: "instrumental" },
        { name: "peaceful" },
        { name: "sad" },
        { name: "piano" },
      ],
    });
    expect(found?.filetype).toBe("mp3");
    expect(found?.tags).toEqual({
      genres: ["ambient"],
      moods: ["calm", "sad"],
      tempo: "slow",
      vocals: "instrumental",
    });
    // Tagged both ways, a track says nothing of its singing.
    expect(
      foundOf({
        ...result("by"),
        tags: [{ name: "instrumental" }, { name: "vocal" }],
      })?.tags.vocals,
    ).toBeNull();
  });

  it("keeps only results under a licence the app can use", () => {
    expect(foundOf(result("by"))?.durationMs).toBe(4200);
    expect(foundOf(result("by-sa"))).toBeNull();
    expect(foundOf(result("by-nc"))).toBeNull();
  });

  it("asks for a new token once when the one kept has lapsed", async () => {
    const asked: boolean[] = [];
    const sent: string[] = [];
    const page = await search(
      searchOf("kind=effect&q=rain"),
      (fresh) => {
        asked.push(fresh);
        return Promise.resolve(fresh ? "new" : "old");
      },
      (_, init) => {
        sent.push(new Headers(init.headers).get("Authorization") ?? "");
        return Promise.resolve(
          sent.length === 1
            ? new Response(null, { status: 401 })
            : Response.json({ page_count: 3, results: [result("cc0")] }),
        );
      },
    );
    expect(asked).toEqual([false, true]);
    expect(sent).toEqual(["Bearer old", "Bearer new"]);
    expect(page).toMatchObject({ found: [{ id: "a1" }], pageCount: 3 });
  });
});
