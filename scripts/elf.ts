// Minimal ELF section-table reader used by the size gate. Parses the section
// headers directly so it works without the Xtensa toolchain installed, and is
// kept free of Deno I/O so the parsing logic is trivially unit-testable.

export type Readable = {
  u16: (offset: number) => number;
  u32: (offset: number) => number;
  u64: (offset: number) => bigint;
};

export type ReaderInput = {
  data: Uint8Array;
  littleEndian: boolean;
};

export const makeReader = ({ data, littleEndian }: ReaderInput): Readable => {
  const view = new DataView(data.buffer, data.byteOffset, data.byteLength);
  return {
    u16: (offset) => {
      return view.getUint16(offset, littleEndian);
    },
    u32: (offset) => {
      return view.getUint32(offset, littleEndian);
    },
    u64: (offset) => {
      return view.getBigUint64(offset, littleEndian);
    },
  };
};

export type SectionSizesInput = {
  data: Uint8Array;
  sections: readonly string[];
};

export const sectionSizes = ({
  data,
  sections,
}: SectionSizesInput): Map<string, number> => {
  const elfClass = data[4]; // 1 = 32-bit, 2 = 64-bit
  const littleEndian = data[5] === 1;
  const byteReader = makeReader({ data, littleEndian });
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
    if (sections.includes(name)) {
      sizes.set(name, size);
    }
  }
  return sizes;
};
