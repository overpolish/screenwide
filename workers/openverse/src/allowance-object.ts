// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// Each address's allowance for the day (`allowance.ts`), kept in a Durable
// Object of its own, named by a hash of the address so the address itself
// is never written down. Its counts are deleted when the day ends. Before
// that, Cloudflare's rate limits stop a burst: from one installation, and
// from one address, which made-up IDs would otherwise get round.

import { DurableObject } from "cloudflare:workers";

import { endOfDay, INSTALL_HEADER, personOf, take } from "./allowance";

export class Allowance extends DurableObject<Env> {
  /** Counts one question from `person`, and says whether it may be sent on
   * to Openverse. */
  async take(person: string): Promise<boolean> {
    const now = Date.now();
    const taken = take(this.ctx.storage.kv, person, now);
    if ((await this.ctx.storage.getAlarm()) === null)
      await this.ctx.storage.setAlarm(endOfDay(now));
    return taken;
  }

  /** The day is over: nothing about it is kept. */
  override async alarm() {
    await this.ctx.storage.deleteAll();
  }
}

const hashOf = async (text: string) =>
  [
    ...new Uint8Array(
      await crypto.subtle.digest("SHA-256", new TextEncoder().encode(text)),
    ),
  ]
    .map((byte) => byte.toString(16).padStart(2, "0"))
    .join("");

/** Whether `request`'s sender may have a question sent on to Openverse now,
 * counting it if so: `busy` where they have asked too often this minute,
 * `usedUp` where today's allowance is gone. */
export async function mayAsk(
  request: Request,
  env: Env,
): Promise<"allowed" | "busy" | "usedUp"> {
  const address = await hashOf(
    request.headers.get("CF-Connecting-IP") ?? "unknown",
  );
  const person = personOf(request.headers.get(INSTALL_HEADER));
  const [byAddress, byPerson] = await Promise.all([
    env.ADDRESS_BURST.limit({ key: address }),
    env.PERSON_BURST.limit({ key: `${address}:${person}` }),
  ]);
  if (!byAddress.success || !byPerson.success) return "busy";
  return (await env.ALLOWANCES.getByName(address).take(person))
    ? "allowed"
    : "usedUp";
}
