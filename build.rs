use bitvec::{field::BitField, prelude::*};
use std::{
    fs, io,
    path::{Path, PathBuf},
};

// Limitations:
// max sprite sheet size: (256 in either direction)
// max number of colors: 4 (for wasm4)
// ASSUMES THE FILE HAS BEEN EXPORTED FROM ASEPRITE IN INDEXED MODE
// does not work otherwise :3 (idk might or might not I didn't test)
//
//
// To force textures to be repacked just save this file
// Otherwise rebuild and it should repack it automatically
fn main() {
    //println!("cargo:rerun-if-changed=./assets");
    println!("cargo:warning=Packing sprites...");

    // Loop over all sprites and pack them
    let dir = std::fs::read_dir("./assets").unwrap();
    for entry in dir.filter_map(|x| x.ok()) {
        println!(
            "cargo:warning=Packing sprite: {}",
            entry.file_name().to_str().unwrap_or_default()
        );
        if let Err(err) = process(&entry.path()) {
            println!("cargo:warning={:?}", err);
        }
    }
}

fn process(path: &Path) -> Result<(), eyre::Error> {
    let file = fs::File::open(path)?;


    let mut decoder = png::Decoder::new(file);
    let header = decoder.read_header_info()?;
    let height = header.height;
    let width = header.width;
    let mut info = decoder.read_info().unwrap();

    // check if there is a transparency metadata file associated with this image file
    let transparency_metadata_path = path.with_file_name(format!("{}_transparency_metadata.json", path.file_name().unwrap().to_str().unwrap()));
    let transparency_metadata_file = fs::File::open(transparency_metadata_path);

    let size = info.output_buffer_size();
    let mut bytes = vec![0u8; size];
    info.next_frame(&mut bytes)?;

    // subtract 1 since aseprite always add a transparent color at index 0
    let palette = &info
        .info()
        .palette
        .as_ref()
        .ok_or(eyre::anyhow!("Not indexed!"))?;
    let palette_count = (palette.len() / 3) - 1; // excluding transparency

    println!("cargo:warning=Width: {}, Height: {}", width, height);
    println!("cargo:warning=Palette Count: {}", palette_count);
    println!("cargo:warning=Buffer Size: {:?}", size);
    println!("cargo:warning=Palette: {:?}", palette);

    if (size as u32) != (width * height) {
        eyre::bail!("Not indexed mode!");
    }

    let bits_per_pixel = palette_count / 2;
    let output_bit_size = (width * height) as usize * bits_per_pixel;

    // Add a header at the start of the file to indicate the width, height, and appropriate flags to use
    // 0..1 byte represent width
    // 1..2 byte represent height
    // 3..4 byte represent bits per pixel (flags)
    const HEADER_BYTE_SIZE: usize = 3;
    const HEADER_BIT_SIZE: usize = HEADER_BYTE_SIZE * 8;
    let mut output_vec = bitvec::bitvec![u8, Msb0; 0; output_bit_size + HEADER_BIT_SIZE];
    output_vec[0..8].store(width);
    output_vec[8..16].store(height);

    // Flags. Either BLIT_1BPP (0) or BLIT_2BPP (1)
    output_vec[16..24].store(bits_per_pixel - 1);

    // Store the bits from the given bytes
    let storage = &mut output_vec[24..];

    // aseprite adds that extra "transparency color"
    // so stupid ngl but wtv we live with it
    for (i, byte) in bytes.into_iter().enumerate() {
        match bits_per_pixel {
            1 => storage.set(i, byte.saturating_sub(1) == 1),
            2 => storage[i * 2..=i * 2 + 1].store(byte.saturating_sub(1)),
            _ => unreachable!(),
        }
    }

    // Create file and save it
    let mut output_path: PathBuf = PathBuf::new();
    output_path.push("./packed");
    output_path.push(path.file_name().unwrap());
    output_path.set_extension("pak");
    let mut output_file = fs::File::create(&output_path)?;
    let bytes = io::copy(&mut output_vec, &mut output_file)?;
    println!(
        "cargo:warning=Saved {bytes} bytes to file {:?}",
        &output_path
    );

    Ok(())
}
