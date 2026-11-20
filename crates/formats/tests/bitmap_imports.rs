use craft_texture::{Image, Kind};
use materialcraft_formats::{BITMAP_EXTENSIONS, BITMAP_FORMATS, Depth, decode, encode};
use std::io::Cursor;

#[test]
fn bitmap_filters_are_specific_and_match_supported_extensions() {
    let extensions: std::collections::BTreeSet<_> = BITMAP_FORMATS.iter().flat_map(|(_, extensions)| extensions.iter().copied()).collect();

    assert_eq!(extensions, BITMAP_EXTENSIONS.iter().copied().collect());

    for excluded in ["svg", "psd", "psb", "surface", "raw", "jng", "wbmp", "*"] {
        assert!(!extensions.contains(excluded));
    }

    assert_eq!(BITMAP_FORMATS[0], ("All formats", BITMAP_EXTENSIONS));
    assert_eq!(materialcraft_formats::VECTOR_FORMATS[0], ("All formats", materialcraft_formats::VECTOR_EXTENSIONS));
}

#[test]
fn common_bitmap_formats_decode() {
    let pixels = Image::filled(4, 4, Kind::Color, [0.25, 0.5, 0.75, 1.0]);
    for extension in ["png", "jpeg", "tga", "bmp", "webp", "exr", "hdr"] {
        let bytes = encode(&pixels, extension, Depth::U8, true).unwrap();
        let image = decode(&bytes, false).unwrap();

        assert_eq!((image.width(), image.height()), (4, 4), "{extension}");
    }

    // GIF and TIFF import only, so another writer makes them
    for format in [image::ImageFormat::Gif, image::ImageFormat::Tiff] {
        let mut bytes = Cursor::new(Vec::new());
        image::RgbaImage::from_pixel(2, 2, image::Rgba([255, 0, 0, 255])).write_to(&mut bytes, format).unwrap();

        assert_eq!(decode(bytes.get_ref(), false).unwrap().width(), 2, "{format:?}");
    }
    assert!(encode(&pixels, "tif", Depth::U8, true).is_err(), "TIFF is not an export format");

    for pgm in [&b"P2\n2 1\n255\n0 255\n"[..], &b"P5\n2 1\n255\n\0\xff"[..]] {
        let image = decode(pgm, false).unwrap();

        assert_eq!(image.kind(), Kind::Gray);
        assert_eq!(image.data(), &[0.0, 1.0]);
    }
}

#[test]
fn dds_supports_compressed_uncompressed_and_float_textures() {
    for format in [
        dds::Format::BC1_UNORM,
        dds::Format::BC3_UNORM,
        dds::Format::BC5_UNORM,
        dds::Format::BC7_UNORM,
        dds::Format::R8G8B8A8_UNORM,
        dds::Format::R32G32B32A32_FLOAT,
    ] {
        let mut bytes = Vec::new();
        let header = dds::header::Header::new_image(4, 4, format);
        let mut encoder = dds::Encoder::new(&mut bytes, format, &header).unwrap();
        let pixels = [64, 128, 192, 255].repeat(16);
        encoder.write_surface(dds::ImageView::new(&pixels, dds::Size::new(4, 4), dds::ColorFormat::RGBA_U8).unwrap()).unwrap();
        encoder.finish().unwrap();
        let image = decode(&bytes, true).unwrap();

        assert_eq!((image.width(), image.height()), (4, 4));
        assert!(image.data().iter().all(|value| value.is_finite()));
        assert!((image.data()[0] - 64.0 / 255.0).abs() < 0.05, "{format:?}");

        if format == dds::Format::R32G32B32A32_FLOAT {
            assert_eq!(decode(&bytes, false).unwrap().data(), image.data(), "float DDS stays linear");
        }

        assert!(decode(&bytes[..bytes.len() / 2], true).is_err());
    }
}

#[test]
fn shared_dds_keeps_rgb_transfer_alpha_and_float_range() {
    let mut bytes = Vec::new();
    dds::header::Header::new_image(1, 1, dds::Format::R8G8B8A8_UNORM).write(&mut bytes).unwrap();
    bytes.extend_from_slice(&[128, 64, 255, 32]);
    let raw = decode(&bytes, true).unwrap();
    let converted = decode(&bytes, false).unwrap();
    for i in 0..3 {
        assert!((converted.data()[i] - craft_texture::srgb_to_linear(raw.data()[i])).abs() < 1e-6);
    }
    assert_eq!(converted.data()[3], raw.data()[3]);

    let mut floats = Vec::new();
    dds::header::Header::new_image(1, 1, dds::Format::R32G32B32A32_FLOAT).write(&mut floats).unwrap();
    for v in [-0.5_f32, 2.0, 0.25, 0.75] {
        floats.extend_from_slice(&v.to_le_bytes());
    }
    for linear in [false, true] {
        assert_eq!(decode(&floats, linear).unwrap().data(), &[-0.5, 2.0, 0.25, 0.75]);
    }
}

#[test]
fn dds_handles_partial_blocks_signed_hdr_swizzles_and_allocation_caps() {
    // Partial (non-4x4-aligned) blocks crop cleanly to the declared dimensions.
    for format in [dds::Format::BC1_UNORM, dds::Format::BC3_UNORM, dds::Format::BC5_UNORM, dds::Format::BC7_UNORM] {
        let mut bytes = Vec::new();
        let header = dds::header::Header::new_image(5, 3, format);
        let mut encoder = dds::Encoder::new(&mut bytes, format, &header).unwrap();
        let pixels = [64, 128, 192, 255].repeat(5 * 3);
        encoder.write_surface(dds::ImageView::new(&pixels, dds::Size::new(5, 3), dds::ColorFormat::RGBA_U8).unwrap()).unwrap();
        encoder.finish().unwrap();

        let image = decode(&bytes, true).unwrap();
        assert_eq!((image.width(), image.height()), (5, 3), "{format:?}");
        assert_eq!(image.data().len(), 5 * 3 * 4, "{format:?}");
        assert!(image.data().iter().all(|value| value.is_finite()), "{format:?}");
        assert!(decode(&bytes[..bytes.len() - 1], true).is_err(), "{format:?} truncated partial block");
    }

    // Swizzled BGRA8 maps channels to RGBA and applies sRGB->linear when `linear` is false.
    let mut bgra = Vec::new();
    dds::header::Header::new_image(1, 1, dds::Format::B8G8R8A8_UNORM).write(&mut bgra).unwrap();
    bgra.extend_from_slice(&[192, 128, 64, 32]);
    let bgra_raw = decode(&bgra, true).unwrap();
    for (actual, expected) in bgra_raw.data().iter().zip([64.0 / 255.0, 128.0 / 255.0, 192.0 / 255.0, 32.0 / 255.0]) {
        assert!((actual - expected).abs() < 1e-6);
    }
    let bgra_linear = decode(&bgra, false).unwrap();
    for i in 0..3 {
        assert!((bgra_linear.data()[i] - craft_texture::srgb_to_linear(bgra_raw.data()[i])).abs() < 1e-6);
    }
    assert_eq!(bgra_linear.data()[3], bgra_raw.data()[3]);

    // Half-float HDR and negative values survive when decoded linearly.
    let mut half_hdr = Vec::new();
    dds::header::Header::new_image(1, 1, dds::Format::R16G16B16A16_FLOAT).write(&mut half_hdr).unwrap();
    for bits in [0xb800u16, 0x4000, 0x3400, 0x3a00] {
        half_hdr.extend_from_slice(&bits.to_le_bytes());
    }
    assert_eq!(decode(&half_hdr, true).unwrap().data(), &[-0.5, 2.0, 0.25, 0.75]);

    // Signed uncompressed, signed BC4/BC5, and unsigned/signed BC6H decode to finite values.
    let mut snorm = Vec::new();
    dds::header::Header::new_image(1, 1, dds::Format::R8G8B8A8_SNORM).write(&mut snorm).unwrap();
    snorm.extend_from_slice(&[(-64i8) as u8, 64, (-127i8) as u8, 127]);
    let snorm_img = decode(&snorm, true).unwrap();
    for (actual, expected) in snorm_img.data().iter().zip([63.0 / 254.0, 191.0 / 254.0, 0.0, 1.0]) {
        assert!((actual - expected).abs() < 1e-4, "actual={actual}, expected={expected}");
    }

    for format in [dds::Format::BC4_SNORM, dds::Format::BC5_SNORM] {
        let mut bytes = Vec::new();
        let header = dds::header::Header::new_image(4, 4, format);
        let mut encoder = dds::Encoder::new(&mut bytes, format, &header).unwrap();
        let pixels: Vec<u8> = [0.5_f32, -0.5, 0.0, 1.0].into_iter().flat_map(f32::to_le_bytes).collect::<Vec<u8>>().repeat(16);
        encoder.write_surface(dds::ImageView::new(&pixels, dds::Size::new(4, 4), dds::ColorFormat::RGBA_F32).unwrap()).unwrap();
        encoder.finish().unwrap();
        let image = decode(&bytes, true).unwrap();
        assert_eq!((image.width(), image.height()), (4, 4), "{format:?}");
        assert!(image.data().iter().all(|value| value.is_finite()), "{format:?}");
    }

    for format in [dds::Format::BC6H_UF16, dds::Format::BC6H_SF16] {
        let mut bytes = Vec::new();
        dds::header::Header::new_image(4, 4, format).write(&mut bytes).unwrap();
        bytes.extend_from_slice(&[0u8; 16]);
        let image = decode(&bytes, true).unwrap();
        assert_eq!((image.width(), image.height()), (4, 4), "{format:?}");
        assert!(image.data().iter().all(|value| value.is_finite()), "{format:?}");
    }

    // Oversized headers are rejected before allocating pixel buffers.
    for (w, h) in [(craft_texture::MAX_SIDE + 1, 1), (1, craft_texture::MAX_SIDE + 1)] {
        let mut oversized = Vec::new();
        dds::header::Header::new_image(w, h, dds::Format::R8G8B8A8_UNORM).write(&mut oversized).unwrap();
        let err = decode(&oversized, true).unwrap_err().to_string();
        assert!(err.contains(&craft_texture::MAX_SIDE.to_string()), "unexpected error for {w}x{h}: {err}");
    }
}
