// Size gate for the ESP32 firmware image.
//
// Fails when the flash-resident sections grow past a budget, so future
// changes cannot silently exceed the OTA partition (1700K) and only break
// at flash time. Parses the ELF section table directly, so it does not
// depend on the Xtensa toolchain being installed.
//
// Usage: deno run --allow-read scripts/check-size.ts [max_flash_bytes]

import map from "lodash/map.js";
import sumBy from "lodash/sumBy.js";
import { z as zod } from "zod";
import { sectionSizes } from "./elf.ts";

const ELF_PATH = "target/xtensa-esp32-espidf/release/esp32-gate-opener";
export const DEFAULT_BUDGET = 1_400_000;
const FLASH_SECTIONS = [".flash.text", ".flash.rodata"] as const;

const budgetSchema = zod.number().int().positive();

export const parseBudget = (raw: string | undefined): number => {
  const parsed = raw === undefined ? DEFAULT_BUDGET : Number.parseInt(raw, 10);
  const result = budgetSchema.safeParse(parsed);
  if (!result.success) {
    throw new Error(`invalid budget: ${raw ?? String(DEFAULT_BUDGET)}`);
  }
  return result.data;
};

const main = (): void => {
  let budget: number;
  try {
    budget = parseBudget(Deno.args[0]);
  } catch (error) {
    console.error(`error: ${(error as Error).message}`);
    Deno.exit(1);
  }

  let data: Uint8Array;
  try {
    data = Deno.readFileSync(ELF_PATH);
  } catch {
    console.error(`error: ${ELF_PATH} not found; build first`);
    Deno.exit(1);
  }

  const sizes = sectionSizes({ data, sections: FLASH_SECTIONS });
  const total = sumBy(
    FLASH_SECTIONS as readonly string[],
    (section: string) => {
      return sizes.get(section) ?? 0;
    },
  );
  const parts = map(FLASH_SECTIONS as readonly string[], (section: string) => {
    return `${section}=${sizes.get(section) ?? 0}`;
  }).join(" ");
  console.log(`${parts} total=${total} budget=${budget}`);

  if (total > budget) {
    console.error(
      `error: firmware grows past the ${budget} byte budget (${total} bytes)`,
    );
    Deno.exit(1);
  }
  console.log("size OK");
};

if (import.meta.main) {
  main();
}
