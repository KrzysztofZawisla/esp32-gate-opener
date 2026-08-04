import { assertEquals, assertThrows } from "@jsr/std__assert";
import { describe, it } from "@jsr/std__testing/bdd";
import { makeReader, sectionSizes } from "./elf.ts";
import { DEFAULT_BUDGET, parseBudget } from "./check-size.ts";

const writeU16 = (view: DataView, offset: number, value: number): void =>
  view.setUint16(offset, value, true);
const writeU32 = (view: DataView, offset: number, value: number): void =>
  view.setUint32(offset, value, true);

// Builds a minimal, valid 32-bit little-endian ELF whose section header table
// lists the given sections plus the required null section and ".shstrtab".
const buildElf32 = (
  sections: Array<{ name: string; size: number }>,
): Uint8Array => {
  const strtabNames = [...sections.map((section) => section.name), ".shstrtab"];
  const strtab = new Uint8Array(
    strtabNames.reduce((length, name) => length + name.length + 1, 0),
  );
  const nameOffsets = new Map<string, number>();
  let stringOffset = 0;
  for (const name of strtabNames) {
    nameOffsets.set(name, stringOffset);
    for (let i = 0; i < name.length; i += 1) {
      strtab[stringOffset + i] = name.charCodeAt(i);
    }
    stringOffset += name.length + 1;
  }

  const headerSize = 52;
  const sectionHeaderSize = 40;
  const sectionCount = sections.length + 2;
  const shstrtabIndex = sectionCount - 1;
  const shstrtabOffset = headerSize;
  const sectionHeaderOffset = shstrtabOffset + strtab.length;

  const elf = new Uint8Array(
    sectionHeaderOffset + sectionCount * sectionHeaderSize,
  );
  const view = new DataView(elf.buffer);

  elf.set([0x7f, 0x45, 0x4c, 0x46]);
  elf[4] = 1; // 32-bit
  elf[5] = 1; // little-endian
  elf[6] = 1;
  writeU16(view, 16, 2); // e_type: EXEC
  writeU16(view, 18, 243); // e_machine: Xtensa
  writeU32(view, 20, 1); // e_version
  writeU32(view, 24, 0); // e_entry
  writeU32(view, 28, 0); // e_phoff
  writeU32(view, 32, sectionHeaderOffset); // e_shoff
  writeU32(view, 36, 0); // e_flags
  writeU16(view, 40, headerSize); // e_ehsize
  writeU16(view, 44, 0); // e_phnum
  writeU16(view, 46, sectionHeaderSize); // e_shentsize
  writeU16(view, 48, sectionCount); // e_shnum
  writeU16(view, 50, shstrtabIndex); // e_shstrndx

  elf.set(strtab, shstrtabOffset);

  const writeSectionHeader = (
    index: number,
    nameOffset: number,
    type: number,
    fileOffset: number,
    size: number,
  ): void => {
    const base = sectionHeaderOffset + index * sectionHeaderSize;
    writeU32(view, base, nameOffset);
    writeU32(view, base + 4, type);
    writeU32(view, base + 8, 0); // sh_flags
    writeU32(view, base + 12, 0); // sh_addr
    writeU32(view, base + 16, fileOffset); // sh_offset
    writeU32(view, base + 20, size); // sh_size
    writeU32(view, base + 24, 0); // sh_link
    writeU32(view, base + 28, 0); // sh_info
    writeU32(view, base + 32, 0); // sh_addralign
    writeU32(view, base + 36, 0); // sh_entsize
  };

  writeSectionHeader(0, 0, 0, 0, 0);

  sections.forEach((section, index) => {
    const nameOffset = nameOffsets.get(section.name);
    if (nameOffset === undefined) {
      throw new Error(`missing name offset for ${section.name}`);
    }
    writeSectionHeader(index + 1, nameOffset, 1, 0, section.size);
  });

  const shstrtabNameOffset = nameOffsets.get(".shstrtab");
  if (shstrtabNameOffset === undefined) {
    throw new Error("missing .shstrtab name offset");
  }
  writeSectionHeader(
    shstrtabIndex,
    shstrtabNameOffset,
    3,
    shstrtabOffset,
    strtab.length,
  );

  return elf;
};

describe("makeReader", () => {
  it("reads big-endian integers", () => {
    const data = new Uint8Array([
      0x12,
      0x34,
      0x56,
      0x78,
      0x9a,
      0xbc,
      0xde,
      0xf0,
    ]);
    const reader = makeReader(data, false);
    assertEquals(reader.u16(0), 0x1234);
    assertEquals(reader.u32(0), 0x12345678);
    assertEquals(reader.u64(0), 0x123456789abcdef0n);
  });

  it("reads little-endian integers", () => {
    const data = new Uint8Array([
      0x12,
      0x34,
      0x56,
      0x78,
      0x9a,
      0xbc,
      0xde,
      0xf0,
    ]);
    const reader = makeReader(data, true);
    assertEquals(reader.u16(0), 0x3412);
    assertEquals(reader.u32(0), 0x78563412);
    assertEquals(reader.u64(0), 0xf0debc9a78563412n);
  });
});

describe("sectionSizes", () => {
  it("returns sizes only for the requested sections", () => {
    const elf = buildElf32([
      { name: ".flash.text", size: 100 },
      { name: ".flash.rodata", size: 250 },
      { name: ".debug", size: 9999 },
    ]);
    const sizes = sectionSizes(elf, [".flash.text", ".flash.rodata"]);
    assertEquals(sizes.get(".flash.text"), 100);
    assertEquals(sizes.get(".flash.rodata"), 250);
    assertEquals(sizes.has(".debug"), false);
  });

  it("omits a requested section that does not exist", () => {
    const elf = buildElf32([{ name: ".flash.text", size: 42 }]);
    const sizes = sectionSizes(elf, [".flash.text", ".flash.rodata"]);
    assertEquals(sizes.get(".flash.text"), 42);
    assertEquals(sizes.has(".flash.rodata"), false);
  });

  it("handles an empty section list", () => {
    const elf = buildElf32([{ name: ".flash.text", size: 42 }]);
    assertEquals(sectionSizes(elf, []).size, 0);
  });
});

describe("parseBudget", () => {
  it("falls back to the default budget when no argument is given", () => {
    assertEquals(parseBudget(undefined), DEFAULT_BUDGET);
  });

  it("parses a valid budget", () => {
    assertEquals(parseBudget("1500000"), 1_500_000);
  });

  it("rejects non-numeric input", () => {
    assertThrows(() => parseBudget("not-a-number"));
  });

  it("rejects negative budgets", () => {
    assertThrows(() => parseBudget("-1000"));
  });
});
