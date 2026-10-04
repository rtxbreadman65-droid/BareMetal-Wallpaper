#![no_std]
#![no_main]

use uefi::prelude::*;
use uefi::Status;
use uefi::table::boot::SearchType;
use uefi::proto::console::gop::GraphicsOutput;
use uefi::Identify;

static WALLPAPER_RAW: &[u8] = include_bytes!("wallpaper.raw");
static WIDTH: usize = 1500;
static HEIGHT: usize = 853;

pub unsafe extern "C" fn display_image(fb_ptr: *mut u32, screen_with: usize, screen_height: usize, screen_stride: usize) {
    unsafe {
        let start_x = (screen_with - WIDTH) / 2;
        let start_y = (screen_height - HEIGHT) / 2;
        let src_ptr = WALLPAPER_RAW.as_ptr() as *const u32;
        for y in 0..HEIGHT {
            let row_offset = (start_y + y) * screen_stride + start_x;
            let dest_row_ptr = fb_ptr.add(row_offset);
            let src_row_ptr = src_ptr.add(y * WIDTH);
            core::ptr::copy_nonoverlapping(src_row_ptr, dest_row_ptr, WIDTH);
        }
    }
}

pub extern "C" fn kernel_main(fb_ptr: *mut u32, screen_with: usize, screen_height: usize, screen_stride: usize, rdsp: usize) -> ! {
    unsafe { display_image(fb_ptr, screen_with, screen_height, screen_stride); }
    loop {}
}

#[entry]
fn efi_main(image_handle: Handle, system_table: SystemTable<Boot>) -> Status {
    let (fb_ptr, screen_with, screen_height, stride) = {
        let searchtype = SearchType::ByProtocol(&GraphicsOutput::GUID);
        let handle = system_table.boot_services().locate_handle_buffer(searchtype).expect("[-] Unable to locate GraphicsOutput handle.\n");
        let frame_buffer = handle[0];
        let mut open_protocol = system_table.boot_services().open_protocol_exclusive::<GraphicsOutput>(frame_buffer).expect("[-] Unable to open exclusive protocol.\n");
        let mode_info = open_protocol.current_mode_info();
        let screen_with = mode_info.resolution().0;
        let screen_height = mode_info.resolution().1;
        let screen_stride = mode_info.stride();
        let fb_ptr = open_protocol.frame_buffer().as_mut_ptr() as *mut u32;
        (fb_ptr, screen_with, screen_height, screen_stride)
    };
    let rdsp = {
        use uefi::table::cfg;
        let entry = system_table.config_table();
        let mut apic_addr = 0;
        for rdsp in entry {
            if rdsp.guid == cfg::ACPI2_GUID{
                apic_addr = rdsp.address as usize
            }
        }
        apic_addr
    };
    let _ = system_table.exit_boot_services(uefi::table::boot::MemoryType::LOADER_DATA);
    kernel_main(fb_ptr, screen_with, screen_height, stride, rdsp);
}

#[panic_handler]
fn panic(_panic: &core::panic::PanicInfo) -> ! {
    loop {}
}
