use std::path::PathBuf;

fn main() {
    let kernel_path = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            p.pop(); p.pop();
            p.push("target");
            p.push("x86_64-catalyst");
            p.push("debug");
            p.push("catalyst-kernel");
            p
        });

    let bios_path = kernel_path.with_extension("img");

    println!("Catalyst OS - BIOS Disk Image Creator");
    
    let mut boot_config = bootloader::BootConfig::default();
    boot_config.frame_buffer.minimum_framebuffer_width = Some(1280);
    boot_config.frame_buffer.minimum_framebuffer_height = Some(720);

    bootloader::BiosBoot::new(&kernel_path)
        .set_boot_config(&boot_config)
        .create_disk_image(&bios_path)
        .expect("Failed to create BIOS disk image");

    println!("SUCCESS");
}
