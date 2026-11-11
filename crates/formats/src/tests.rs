use super::*;
use materialcraft_image::{Image, Kind, srgb_to_linear};
use std::io::Cursor;

#[test]
fn export_every_format() {
    let img = Image::generate(8, 8, Kind::Color, |x, y| [x as f32 / 7.0, y as f32 / 7.0, 0.5, 1.0]);
    let gray = img.to_gray();
    for (ext, _) in EXPORT_FORMATS {
        for depth in [Depth::U8, Depth::U16, Depth::F32] {
            for i in [&img, &gray] {
                let b = encode(i, ext, depth, true).unwrap_or_else(|e| panic!("{ext}: {e}"));
                let back = decode(&b, false).unwrap_or_else(|e| panic!("{ext} decode: {e}"));
                assert_eq!((back.w, back.h), (8, 8));
            }
        }
    }
    let png16 = encode(&gray, "png", Depth::U16, false).unwrap();
    assert!((decode(&png16, false).unwrap().get(7, 0, false)[0] - gray.get(7, 0, false)[0]).abs() < 1e-3);
}

#[test]
fn names_are_safe() {
    assert_eq!(output_name("$(graph)_$(usage)", "Bricks", "out", "baseColor"), "Bricks_baseColor");
    assert_eq!(safe_file_name("../../etc/passwd"), "passwd");
    assert_eq!(safe_file_name(".."), "resource");
}

#[test]
fn oversized_images_are_refused_before_decoding() {
    let mut png = Cursor::new(Vec::new());
    ::image::GrayImage::new(8193, 1).write_to(&mut png, ::image::ImageFormat::Png).unwrap();
    let err = decode(png.get_ref(), false).unwrap_err().to_string();
    assert!(err.contains("8192"), "{err}");
}

#[test]
fn tga_still_decodes() {
    let mut tga = Cursor::new(Vec::new());
    ::image::RgbaImage::new(2, 2).write_to(&mut tga, ::image::ImageFormat::Tga).unwrap();
    let img = decode(tga.get_ref(), false).unwrap();
    assert_eq!((img.w, img.h, img.kind), (2, 2, Kind::Color));
}

#[test]
fn decode_matches_previous_values() {
    let mut png = Cursor::new(Vec::new());
    ::image::RgbImage::from_pixel(2, 2, ::image::Rgb([128, 128, 128])).write_to(&mut png, ::image::ImageFormat::Png).unwrap();
    let img = decode(png.get_ref(), false).unwrap();
    let want = srgb_to_linear(128.0 / 255.0);
    assert!((img.get(1, 1, false)[0] - want).abs() < 1e-4);
}

#[test]
fn safe_file_name_blocks_windows_device_names() {
    for (name, want) in [
        ("CON", "_CON"),
        ("nul.png", "_nul.png"),
        ("com1", "_com1"),
        ("LPT9.txt", "_LPT9.txt"),
        ("aux. ", "_aux"),
        ("con.", "_con"),
        ("COM0", "COM0"),
        ("console", "console"),
    ] {
        assert_eq!(safe_file_name(name), want, "{name}");
    }
    assert_eq!(safe_file_name("name. . "), "name");
    assert_eq!(safe_file_name("...").as_str(), "resource");
    assert_eq!(safe_file_name(&"a".repeat(5000)).len(), 100);
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn read_capped_bounds_size_and_kind() {
    let dir = std::env::temp_dir().join(format!("mc-read-capped-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("f.bin");
    std::fs::write(&file, b"12345").unwrap();
    assert_eq!(read_capped(&file, 5).unwrap(), b"12345");
    assert_eq!(read_capped(&file, u64::MAX).unwrap(), b"12345", "an unbounded cap must not overflow");
    assert!(read_capped(&file, 4).is_err());
    assert!(read_capped(&dir, 100).is_err(), "directories are not regular files");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn project_archive_entry_count_is_bounded() {
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let stored = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    for i in 0..100_001 {
        zip.start_file(format!("f{i}"), stored).unwrap();
    }
    let bytes = zip.finish().unwrap().into_inner();
    let error = project::read(&bytes, "x").unwrap_err().to_string();
    assert!(error.contains("too many entries"), "{error}");
}

#[test]
fn a_long_name_keeps_its_extension_and_stays_distinct() {
    let long = format!("{}.png", "a".repeat(120));
    let cut = safe_file_name(&long);
    assert!(cut.ends_with(".png") && cut.chars().count() <= 100, "{cut}");
    assert_eq!(cut, safe_file_name(&long), "stable");
    // two names sharing their first 100 characters stay apart
    let (a, b) = (safe_file_name(&format!("{}x.png", "p".repeat(100))), safe_file_name(&format!("{}y.png", "p".repeat(100))));
    assert_ne!(a, b);
    assert!(a.ends_with(".png") && b.ends_with(".png"));
    // a reserved name is still prefixed, and a short name is untouched
    assert_eq!(safe_file_name("ok.png"), "ok.png");
}
