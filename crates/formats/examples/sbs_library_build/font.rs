//! Original monospaced segment outlines, serialized as a small deterministic TrueType font.
//! Intended for the numeric atlas in the library; unrelated characters use a visible fallback.
use super::Result;
use std::collections::BTreeMap;

fn u16be(out: &mut Vec<u8>, value: u16) {
    out.extend(value.to_be_bytes());
}
fn i16be(out: &mut Vec<u8>, value: i16) {
    out.extend(value.to_be_bytes());
}
fn u32be(out: &mut Vec<u8>, value: u32) {
    out.extend(value.to_be_bytes());
}

fn checksum(data: &[u8]) -> u32 {
    data.chunks(4).fold(0_u32, |sum, chunk| {
        let mut bytes = [0; 4];
        for (to, from) in bytes.iter_mut().zip(chunk) {
            *to = *from;
        }
        sum.wrapping_add(u32::from_be_bytes(bytes))
    })
}

fn segments(character: char) -> &'static [usize] {
    match character.to_ascii_uppercase() {
        ' ' => &[],
        '0' | 'O' => &[0, 1, 2, 3, 4, 5],
        '1' => &[1, 2],
        '2' | 'Z' => &[0, 1, 6, 4, 3],
        '3' => &[0, 1, 6, 2, 3],
        '4' => &[5, 6, 1, 2],
        '5' | 'S' => &[0, 5, 6, 2, 3],
        '6' => &[0, 5, 4, 3, 2, 6],
        '7' => &[0, 1, 2],
        '8' | 'B' => &[0, 1, 2, 3, 4, 5, 6],
        '9' => &[0, 1, 2, 3, 5, 6],
        'A' => &[0, 1, 2, 4, 5, 6],
        'C' => &[0, 3, 4, 5],
        'D' => &[0, 1, 2, 3, 11, 12],
        'E' => &[0, 3, 4, 5, 6],
        'F' => &[0, 4, 5, 6],
        'G' => &[0, 2, 3, 4, 5, 6],
        'H' => &[1, 2, 4, 5, 6],
        'I' => &[0, 3, 11, 12],
        'J' => &[1, 2, 3, 4],
        'K' => &[4, 5, 8, 10],
        'L' => &[3, 4, 5],
        'M' => &[1, 2, 4, 5, 7, 8],
        'N' => &[1, 2, 4, 5, 7, 10],
        'P' => &[0, 1, 4, 5, 6],
        'Q' => &[0, 1, 2, 3, 4, 5, 10],
        'R' => &[0, 1, 4, 5, 6, 10],
        'T' => &[0, 11, 12],
        'U' => &[1, 2, 3, 4, 5],
        'V' => &[5, 1, 9, 10],
        'W' => &[1, 2, 4, 5, 9, 10],
        'X' => &[7, 8, 9, 10],
        'Y' => &[7, 8, 12],
        '.' | ',' => &[13],
        '-' | '_' | '=' => &[6],
        '+' => &[6, 11, 12],
        '/' => &[8, 9],
        '\\' => &[7, 10],
        '(' | '[' => &[0, 3, 4, 5],
        ')' | ']' => &[0, 1, 2, 3],
        _ => &[0, 1, 6, 12, 13],
    }
}

fn glyph(character: char) -> Result<Vec<u8>> {
    const LINES: [[f32; 4]; 14] = [
        [100., 700., 500., 700.],
        [500., 700., 500., 400.],
        [500., 400., 500., 100.],
        [100., 100., 500., 100.],
        [100., 400., 100., 100.],
        [100., 700., 100., 400.],
        [100., 400., 500., 400.],
        [100., 700., 300., 400.],
        [500., 700., 300., 400.],
        [100., 100., 300., 400.],
        [500., 100., 300., 400.],
        [300., 700., 300., 400.],
        [300., 400., 300., 100.],
        [280., 70., 320., 70.],
    ];
    let segments = segments(character);
    let mut points = Vec::<[i16; 2]>::new();
    for index in segments {
        let [ax, ay, bx, by] = *LINES.get(*index).ok_or("invalid glyph segment")?;
        let length = (bx - ax).hypot(by - ay).max(1.0);
        let (nx, ny) = (-(by - ay) / length * 25.0, (bx - ax) / length * 25.0);
        for (x, y) in [(ax + nx, ay + ny), (bx + nx, by + ny), (bx - nx, by - ny), (ax - nx, ay - ny)] {
            points.push([x.round() as i16, y.round() as i16]);
        }
    }
    let mut out = Vec::new();
    i16be(&mut out, segments.len() as i16);
    for value in [
        points.iter().map(|p| p[0]).min().unwrap_or(0),
        points.iter().map(|p| p[1]).min().unwrap_or(0),
        points.iter().map(|p| p[0]).max().unwrap_or(0),
        points.iter().map(|p| p[1]).max().unwrap_or(0),
    ] {
        i16be(&mut out, value);
    }
    for index in 0..segments.len() {
        u16be(&mut out, (index * 4 + 3) as u16);
    }
    u16be(&mut out, 0); // No hinting instructions.
    out.extend(std::iter::repeat_n(1_u8, points.len())); // On-curve points, signed 16-bit deltas.
    for axis in 0..2 {
        let mut previous = 0;
        for point in &points {
            i16be(&mut out, point[axis] - previous);
            previous = point[axis];
        }
    }
    Ok(out)
}

pub fn generate() -> Result<Vec<u8>> {
    const COUNT: u16 = 96; // .notdef plus printable ASCII.
    let mut tables = BTreeMap::<[u8; 4], Vec<u8>>::new();
    let mut glyf = Vec::new();
    let mut loca = Vec::new();
    let mut hmtx = Vec::new();
    for character in std::iter::once('?').chain((32_u8..=126).map(char::from)) {
        u32be(&mut loca, glyf.len() as u32);
        let bytes = glyph(character)?;
        let bearing = i16::from_be_bytes([*bytes.get(2).ok_or("missing glyph bound")?, *bytes.get(3).ok_or("missing glyph bound")?]);
        glyf.extend(bytes);
        while glyf.len() % 4 != 0 {
            glyf.push(0);
        }
        u16be(&mut hmtx, 600);
        i16be(&mut hmtx, bearing);
    }
    u32be(&mut loca, glyf.len() as u32);
    tables.insert(*b"glyf", glyf);
    tables.insert(*b"loca", loca);
    tables.insert(*b"hmtx", hmtx);
    let mut head = Vec::new();
    for value in [0x00010000, 0x00010000, 0, 0x5f0f3cf5] {
        u32be(&mut head, value);
    }
    u16be(&mut head, 3);
    u16be(&mut head, 1000);
    head.extend([0; 16]); // Unspecified fixed creation/modification timestamps.
    for value in [50, 0, 550, 750, 0, 8, 2, 1, 0] {
        i16be(&mut head, value);
    }
    tables.insert(*b"head", head);
    let mut hhea = Vec::new();
    u32be(&mut hhea, 0x00010000);
    for value in [800, -200, 200, 600, 0, 50, 550, 1, 0, 0, 0, 0, 0, 0, 0] {
        i16be(&mut hhea, value);
    }
    u16be(&mut hhea, COUNT);
    tables.insert(*b"hhea", hhea);
    let mut maxp = Vec::new();
    u32be(&mut maxp, 0x00010000);
    for value in [COUNT, 32, 8, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0] {
        u16be(&mut maxp, value);
    }
    tables.insert(*b"maxp", maxp);
    let mut os2 = Vec::new();
    for value in [0, 600, 400, 5, 0, 650, 600, 0, 75, 650, 600, 0, 350, 50, 300, 0] {
        u16be(&mut os2, value);
    }
    os2.extend([0; 10]);
    for value in [1, 0, 0, 0] {
        u32be(&mut os2, value);
    }
    os2.extend(b"MCFT");
    for value in [0x40, 32, 126, 800, (-200_i16) as u16, 200, 800, 200] {
        u16be(&mut os2, value);
    }
    tables.insert(*b"OS/2", os2);
    let mut cmap = Vec::new();
    for value in [0, 1, 3, 1] {
        u16be(&mut cmap, value);
    }
    u32be(&mut cmap, 12);
    // One contiguous ASCII range and the mandatory sentinel range.
    for value in [4, 32, 0, 4, 4, 1, 0, 126, 0xffff, 0, 32, 0xffff, (-31_i16) as u16, 1, 0, 0] {
        u16be(&mut cmap, value);
    }
    tables.insert(*b"cmap", cmap);
    let strings = [
        (1, "MaterialCraft Grid"),
        (2, "Regular"),
        (3, "MaterialCraftGrid-1"),
        (4, "MaterialCraft Grid Regular"),
        (5, "Version 1.0"),
        (6, "MaterialCraftGrid-Regular"),
    ];
    let mut name = Vec::new();
    u16be(&mut name, 0);
    u16be(&mut name, strings.len() as u16);
    u16be(&mut name, (6 + strings.len() * 12) as u16);
    let mut storage = Vec::new();
    for (id, string) in strings {
        let bytes: Vec<_> = string.encode_utf16().flat_map(u16::to_be_bytes).collect();
        for value in [3, 1, 0x409, id, bytes.len() as u16, storage.len() as u16] {
            u16be(&mut name, value);
        }
        storage.extend(bytes);
    }
    name.extend(storage);
    tables.insert(*b"name", name);
    let mut post = Vec::new();
    u32be(&mut post, 0x00030000);
    u32be(&mut post, 0);
    i16be(&mut post, -100);
    i16be(&mut post, 50);
    for value in [1, 0, 0, 0, 0] {
        u32be(&mut post, value);
    }
    tables.insert(*b"post", post);
    let count = tables.len() as u16;
    let power = 1_u16 << (15 - count.leading_zeros());
    let mut out = Vec::new();
    u32be(&mut out, 0x00010000);
    for value in [count, power * 16, power.trailing_zeros() as u16, count * 16 - power * 16] {
        u16be(&mut out, value);
    }
    let mut payload = Vec::new();
    let mut head_offset = None;
    for (tag, table) in tables {
        let offset = 12 + usize::from(count) * 16 + payload.len();
        if tag == *b"head" {
            head_offset = Some(offset);
        }
        out.extend(tag);
        u32be(&mut out, checksum(&table));
        u32be(&mut out, offset as u32);
        u32be(&mut out, table.len() as u32);
        payload.extend(table);
        while payload.len() % 4 != 0 {
            payload.push(0);
        }
    }
    out.extend(payload);
    let adjustment = 0xb1b0afba_u32.wrapping_sub(checksum(&out));
    let position = head_offset.ok_or("font head table missing")? + 8;
    out.get_mut(position..position + 4).ok_or("font checksum position missing")?.copy_from_slice(&adjustment.to_be_bytes());
    Ok(out)
}
