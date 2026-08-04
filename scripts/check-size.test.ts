import { assertEquals, assertThrows } from "@jsr/std__assert";
import { DEFAULT_BUDGET, parseBudget } from "./check-size.ts";
import { makeReader, sectionSizes } from "./elf.ts";

type WriterInput = {
  view: DataView;
  offset: number;
  value: number;
};

const writeU16 = ({ view, offset, value }: WriterInput): void => {
  view.setUint16(offset, value, true);
};
const writeU32 = ({ view, offset, value }: WriterInput): void => {
  view.setUint32(offset, value, true);
};

// Builds a minimal, valid 32-bit little-endian ELF whose section header table
// lists the given sections plus the required null section and ".shstrtab".
const buildElf32 = (
  sections: Array<{ name: string; size: number }>,
): Uint8Array => {
  const strtabNames = [
    ...sections.map((section) => {
      return section.name;
    }),
    ".shstrtab",
  ];
  let totalLength = 0;
  for (const name of strtabNames) {
    totalLength += name.length + 1;
  }
  const strtab = new Uint8Array(totalLength);

  const nameOffsets = new Map<string, number>();
  let stringOffset = 0;
  for (const name of strtabNames) {
    nameOffsets.set(name, stringOffset);
    for (let index = 0; index < name.length; index += 1) {
      strtab[stringOffset + index] = name.charCodeAt(index);
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
  writeU16({ view, offset: 16, value: 2 }); // e_type: EXEC
  writeU16({ view, offset: 18, value: 243 }); // e_machine: Xtensa
  writeU32({ view, offset: 20, value: 1 }); // e_version
  writeU32({ view, offset: 24, value: 0 }); // e_entry
  writeU32({ view, offset: 28, value: 0 }); // e_phoff
  writeU32({ view, offset: 32, value: sectionHeaderOffset }); // e_shoff
  writeU32({ view, offset: 36, value: 0 }); // e_flags
  writeU16({ view, offset: 40, value: headerSize }); // e_ehsize
  writeU16({ view, offset: 44, value: 0 }); // e_phnum
  writeU16({ view, offset: 46, value: sectionHeaderSize }); // e_shentsize
  writeU16({ view, offset: 48, value: sectionCount }); // e_shnum
  writeU16({ view, offset: 50, value: shstrtabIndex }); // e_shstrndx

  elf.set(strtab, shstrtabOffset);

  type SectionHeaderInput = {
    index: number;
    nameOffset: number;
    type: number;
    fileOffset: number;
    size: number;
  };
  const writeSectionHeader = ({
    index,
    nameOffset,
    type,
    fileOffset,
    size,
  }: SectionHeaderInput): void => {
    const base = sectionHeaderOffset + index * sectionHeaderSize;
    writeU32({ view, offset: base, value: nameOffset });
    writeU32({ view, offset: base + 4, value: type });
    writeU32({ view, offset: base + 8, value: 0 }); // sh_flags
    writeU32({ view, offset: base + 12, value: 0 }); // sh_addr
    writeU32({ view, offset: base + 16, value: fileOffset }); // sh_offset
    writeU32({ view, offset: base + 20, value: size }); // sh_size
    writeU32({ view, offset: base + 24, value: 0 }); // sh_link
    writeU32({ view, offset: base + 28, value: 0 }); // sh_info
    writeU32({ view, offset: base + 32, value: 0 }); // sh_addralign
    writeU32({ view, offset: base + 36, value: 0 }); // sh_entsize
  };

  writeSectionHeader({
    index: 0,
    nameOffset: 0,
    type: 0,
    fileOffset: 0,
    size: 0,
  });

  for (let index = 0; index < sections.length; index += 1) {
    const section = sections[index];
    const nameOffset = nameOffsets.get(section.name);
    if (nameOffset === undefined) {
      throw new Error(`missing name offset for ${section.name}`);
    }
    writeSectionHeader({
      index: index + 1,
      nameOffset,
      type: 1,
      fileOffset: 0,
      size: section.size,
    });
  }

  const shstrtabNameOffset = nameOffsets.get(".shstrtab");
  if (shstrtabNameOffset === undefined) {
    throw new Error("missing .shstrtab name offset");
  }
  writeSectionHeader({
    index: shstrtabIndex,
    nameOffset: shstrtabNameOffset,
    type: 3,
    fileOffset: shstrtabOffset,
    size: strtab.length,
  });

  return elf;
};

Deno.test("makeReader reads big-endian integers", () => {
  const data = new Uint8Array([0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0]);
  const reader = makeReader({ data, littleEndian: false });
  assertEquals(reader.u16(0), 0x1234);
  assertEquals(reader.u32(0), 0x12345678);
  assertEquals(reader.u64(0), 0x123456789abcdef0n);
});

Deno.test("makeReader reads little-endian integers", () => {
  const data = new Uint8Array([0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0]);
  const reader = makeReader({ data, littleEndian: true });
  assertEquals(reader.u16(0), 0x3412);
  assertEquals(reader.u32(0), 0x78563412);
  assertEquals(reader.u64(0), 0xf0debc9a78563412n);
});

Deno.test("sectionSizes returns sizes only for the requested sections", () => {
  const elf = buildElf32([
    { name: ".flash.text", size: 100 },
    { name: ".flash.rodata", size: 250 },
    { name: ".debug", size: 9999 },
  ]);
  const sizes = sectionSizes({
    data: elf,
    sections: [".flash.text", ".flash.rodata"],
  });
  assertEquals(sizes.get(".flash.text"), 100);
  assertEquals(sizes.get(".flash.rodata"), 250);
  assertEquals(sizes.has(".debug"), false);
});

Deno.test("sectionSizes omits a requested section that does not exist", () => {
  const elf = buildElf32([{ name: ".flash.text", size: 42 }]);
  const sizes = sectionSizes({
    data: elf,
    sections: [".flash.text", ".flash.rodata"],
  });
  assertEquals(sizes.get(".flash.text"), 42);
  assertEquals(sizes.has(".flash.rodata"), false);
});

Deno.test("sectionSizes handles an empty section list", () => {
  const elf = buildElf32([{ name: ".flash.text", size: 42 }]);
  assertEquals(sectionSizes({ data: elf, sections: [] }).size, 0);
});

Deno.test(
  "parseBudget falls back to the default budget when no argument is given",
  () => {
    assertEquals(parseBudget(undefined), DEFAULT_BUDGET);
  },
);

Deno.test("parseBudget parses a valid budget", () => {
  assertEquals(parseBudget("1500000"), 1_500_000);
});

Deno.test("parseBudget rejects non-numeric input", () => {
  assertThrows(() => {
    parseBudget("not-a-number");
  });
});

Deno.test("parseBudget rejects negative budgets", () => {
  assertThrows(() => {
    parseBudget("-1000");
  });
});
