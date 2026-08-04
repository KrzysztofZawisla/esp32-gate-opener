// Size gate for the ESP32 firmware image.
//
// Fails when the flash-resident sections grow past a budget, so future
// changes cannot silently exceed the OTA partition (1700K) and only break
// at flash time. Parses the ELF section table directly, so it does not
// depend on the Xtensa toolchain being installed.
//
// Usage: deno run --allow-read scripts/check-size.ts [max_flash_bytes]

const ELF_PATH = "target/xtensa-esp32-espidf/release/esp32-gate-opener";
const DEFAULT_BUDGET = 1_400_000;
const FLASH_SECTIONS = [".flash.text", ".flash.rodata"] as const;

type Readable = {
  u16: (offset: number) => number;
  u32: (offset: number) => number;
  u64: (offset: number) => bigint;
};

const makeReader = (data: Uint8Array, littleEndian: boolean): Readable => {
  const view = new DataView(data.buffer, data.byteOffset, data.byteLength);
  return {
    u16: (offset) => view.getUint16(offset, littleEndian),
    u32: (offset) => view.getUint32(offset, littleEndian),
    u64: (offset) => view.getBigUint64(offset, littleEndian),
  };
};

const sectionSizes = (data: Uint8Array): Map<string, number> => {
  const elfClass = data[4]; // 1 = 32-bit, 2 = 64-bit
  const littleEndian = data[5] === 1;
  const byteReader = makeReader(data, littleEndian);
  const is64 = elfClass === 2;

  const sectionHeaderOffset = is64
    ? Number(byteReader.u64(40))
    : byteReader.u32(32);
  const sectionHeaderEntrySize = byteReader.u16(46);
  const sectionHeaderCount = byteReader.u16(48);
  const sectionHeaderStringTableIndex = byteReader.u16(50);

  const sectionHeaderStringTableOffset = is64
    ? Number(
      byteReader.u64(
        sectionHeaderOffset +
          sectionHeaderStringTableIndex * sectionHeaderEntrySize +
          24,
      ),
    )
    : byteReader.u32(
      sectionHeaderOffset +
        sectionHeaderStringTableIndex * sectionHeaderEntrySize +
        16,
    );

  const decoder = new TextDecoder();
  const sectionName = (nameOffset: number): string => {
    let end = nameOffset;
    while (data[sectionHeaderStringTableOffset + end] !== 0) end += 1;
    return decoder.decode(
      data.subarray(
        sectionHeaderStringTableOffset + nameOffset,
        sectionHeaderStringTableOffset + end,
      ),
    );
  };

  const sizes = new Map<string, number>();
  for (let index = 0; index < sectionHeaderCount; index += 1) {
    const base = sectionHeaderOffset + index * sectionHeaderEntrySize;
    const nameOffset = byteReader.u32(base);
    const size = is64
      ? Number(byteReader.u64(base + 32))
      : byteReader.u32(base + 20);
    const name = sectionName(nameOffset);
    if ((FLASH_SECTIONS as readonly string[]).includes(name)) {
      sizes.set(name, size);
    }
  }
  return sizes;
};

const main = (): void => {
  const budget = Deno.args[0]
    ? Number.parseInt(Deno.args[0], 10)
    : DEFAULT_BUDGET;
  if (!Number.isFinite(budget)) {
    console.error(`error: invalid budget: ${Deno.args[0]}`);
    Deno.exit(1);
  }

  let data: Uint8Array;
  try {
    data = Deno.readFileSync(ELF_PATH);
  } catch {
    console.error(`error: ${ELF_PATH} not found; build first`);
    Deno.exit(1);
  }

  const sizes = sectionSizes(data);
  const total = FLASH_SECTIONS.reduce(
    (accumulator, section) => accumulator + (sizes.get(section) ?? 0),
    0,
  );
  const parts = FLASH_SECTIONS.map(
    (section) => `${section}=${sizes.get(section) ?? 0}`,
  ).join(" ");
  console.log(`${parts} total=${total} budget=${budget}`);

  if (total > budget) {
    console.error(
      `error: firmware grows past the ${budget} byte budget (${total} bytes)`,
    );
    Deno.exit(1);
  }
  console.log("size OK");
};

main();
