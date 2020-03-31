#[cfg(target_family = "unix")]
pub fn commit(size: usize, executable: bool) -> *mut u8 {
    let mut prot = libc::PROT_READ | libc::PROT_WRITE;

    if executable {
        prot |= libc::PROT_EXEC;
    }

    let ptr = unsafe {
        libc::mmap(
            std::ptr::null_mut(),
            size,
            prot,
            libc::MAP_PRIVATE | libc::MAP_ANON,
            -1,
            0,
        )
    };

    if ptr == libc::MAP_FAILED {
        panic!("committing memory with mmap() failed");
    }

    ptr as *mut u8
}

#[cfg(target_family = "windows")]
pub fn commit(size: usize, executable: bool) -> *mut u8 {
    use kernel32::VirtualAlloc;
    use winapi::um::winnt::{MEM_COMMIT, MEM_RESERVE, PAGE_EXECUTE_READWRITE, PAGE_READWRITE};

    let prot = if executable {
        PAGE_EXECUTE_READWRITE
    } else {
        PAGE_READWRITE
    };

    let ptr = unsafe {
        VirtualAlloc(
            std::ptr::null_mut(),
            size as u64,
            MEM_COMMIT | MEM_RESERVE,
            prot,
        )
    };

    if ptr.is_null() {
        panic!("VirtualAlloc failed");
    }

    ptr as *mut u8
}

#[cfg(target_family = "unix")]
pub fn protect(start: *mut u8, size: usize, access: Access) {
    if access.is_none() {
        discard(start, size);
        return;
    }

    let protection = match access {
        Access::None => unreachable!(),
        Access::Read => libc::PROT_READ,
        Access::ReadWrite => libc::PROT_READ | libc::PROT_WRITE,
        Access::ReadExecutable => libc::PROT_READ | libc::PROT_EXEC,
        Access::ReadWriteExecutable => libc::PROT_READ | libc::PROT_WRITE | libc::PROT_EXEC,
    };

    let res = unsafe { libc::mprotect(start.cast(), size, protection) };

    if res != 0 {
        panic!("mprotect() failed");
    }
}

#[cfg(target_family = "windows")]
pub fn protect(start: *mut u8, size: usize, access: Access) {
    debug_assert!(start.is_page_aligned());
    debug_assert!(mem::is_page_aligned(size));

    use kernel32::VirtualAlloc;
    use winapi::um::winnt::{
        MEM_COMMIT, PAGE_EXECUTE_READ, PAGE_EXECUTE_READWRITE, PAGE_READONLY, PAGE_READWRITE,
    };

    if access.is_none() {
        discard(start, size);
        return;
    }

    let protection = match access {
        Access::None => unreachable!(),
        Access::Read => PAGE_READONLY,
        Access::ReadWrite => PAGE_READWRITE,
        Access::ReadExecutable => PAGE_EXECUTE_READ,
        Access::ReadWriteExecutable => PAGE_EXECUTE_READWRITE,
    };

    let ptr = unsafe { VirtualAlloc(start.to_mut_ptr(), size as u64, MEM_COMMIT, protection) };

    if ptr.is_null() {
        panic!("VirtualAlloc failed");
    }
}

pub enum Access {
    None,
    Read,
    ReadWrite,
    ReadExecutable,
    ReadWriteExecutable,
}

impl Access {
    fn is_none(&self) -> bool {
        match self {
            Access::None => true,
            _ => false,
        }
    }
}

#[cfg(target_family = "unix")]
pub fn discard(ptr: *mut u8, size: usize) {
    let res = unsafe { libc::madvise(ptr.cast(), size, libc::MADV_DONTNEED) };

    if res != 0 {
        panic!("discarding memory with madvise() failed");
    }

    let res = unsafe { libc::mprotect(ptr.cast(), size, libc::PROT_NONE) };

    if res != 0 {
        panic!("discarding memory with mprotect() failed");
    }
}

#[cfg(target_family = "windows")]
pub fn discard(ptr: *mut u8, size: usize) {
    use kernel32::VirtualFree;
    use winapi::um::winnt::MEM_DECOMMIT;

    let _ = unsafe { VirtualFree(ptr.cast(), size as u64, MEM_DECOMMIT) };
}

static mut PAGE_SIZE: usize = 0;
static mut PAGE_SIZE_BITS: usize = 0;

pub fn page_size() -> usize {
    let result = unsafe { PAGE_SIZE };

    if result != 0 {
        return result;
    }

    init_page_size();

    unsafe { PAGE_SIZE }
}

pub fn page_size_bits() -> usize {
    let result = unsafe { PAGE_SIZE_BITS };

    if result != 0 {
        return result;
    }

    init_page_size();

    unsafe { PAGE_SIZE_BITS }
}

fn init_page_size() {
    unsafe {
        PAGE_SIZE = determine_page_size();
        assert!((PAGE_SIZE & (PAGE_SIZE - 1)) == 0);

        PAGE_SIZE_BITS = log2(PAGE_SIZE);
    }
}

#[cfg(target_family = "unix")]
fn determine_page_size() -> usize {
    let val = unsafe { libc::sysconf(libc::_SC_PAGESIZE) };

    if val <= 0 {
        panic!("could not determine page size.");
    }

    val as usize
}

#[cfg(target_family = "windows")]
fn determine_page_size() -> usize {
    use winapi::um::sysinfoapi::{GetSystemInfo, LPSYSTEM_INFO, SYSTEM_INFO};

    unsafe {
        let mut system_info: SYSTEM_INFO = std::mem::zeroed();
        GetSystemInfo(&mut system_info as LPSYSTEM_INFO);

        system_info.dwPageSize as usize
    }
}

/// determine log_2 of given value
fn log2(mut val: usize) -> usize {
    let mut log = 0;
    assert!(val <= u32::max_value() as usize);

    if (val & 0xFFFF0000) != 0 {
        val >>= 16;
        log += 16;
    }
    if val >= 256 {
        val >>= 8;
        log += 8;
    }
    if val >= 16 {
        val >>= 4;
        log += 4;
    }
    if val >= 4 {
        val >>= 2;
        log += 2;
    }

    log + (val >> 1)
}

#[test]
fn test_log2() {
    for i in 0..32 {
        assert_eq!(i, log2(1 << i));
    }
}
use std::i32;
use std::mem::size_of;
/// return pointer width: either 4 or 8
/// (although only 64bit architectures are supported right now)
#[inline(always)]
pub fn ptr_width() -> i32 {
    size_of::<*const u8>() as i32
}

#[inline(always)]
pub fn ptr_width_usize() -> usize {
    size_of::<*const u8>() as usize
}

/// returns true if given value is a multiple of a page size.
pub fn is_page_aligned(val: usize) -> bool {
    let align = page_size_bits();

    // we can use shifts here since we know that
    // page size is power of 2
    val == ((val >> align) << align)
}

#[test]
fn test_is_page_aligned() {
    let p = page_size();

    assert_eq!(false, is_page_aligned(1));
    assert_eq!(false, is_page_aligned(2));
    assert_eq!(false, is_page_aligned(64));
    assert_eq!(true, is_page_aligned(p));
    assert_eq!(true, is_page_aligned(2 * p));
    assert_eq!(true, is_page_aligned(3 * p));
}

/// round the given value up to the nearest multiple of a page
pub fn page_align(val: usize) -> usize {
    let align = page_size_bits();

    // we know that page size is power of 2, hence
    // we can use shifts instead of expensive division
    ((val + (1 << align) - 1) >> align) << align
}

#[test]
fn test_page_align() {
    let p = page_size();

    assert_eq!(p, page_align(1));
    assert_eq!(p, page_align(p - 1));
    assert_eq!(p, page_align(p));
    assert_eq!(2 * p, page_align(p + 1));
}

/// rounds the given value `val` up to the nearest multiple
/// of `align`
pub fn align(value: u32, align: u32) -> u32 {
    if align == 0 {
        return value;
    }

    ((value + align - 1) / align) * align
}

/// rounds the given value `val` up to the nearest multiple
/// of `align`
pub fn align_i32(value: i32, align: i32) -> i32 {
    if align == 0 {
        return value;
    }

    ((value + align - 1) / align) * align
}

/// rounds the given value `val` up to the nearest multiple
/// of `align`.
pub fn align_usize(value: usize, align: usize) -> usize {
    if align == 0 {
        return value;
    }

    ((value + align - 1) / align) * align
}

/// returns 'true' if th given `value` is already aligned
/// to `align`.
pub fn is_aligned(value: usize, align: usize) -> bool {
    align_usize(value, align) == value
}

/// returns true if value fits into u8 (unsigned 8bits).
pub fn fits_u8(value: i64) -> bool {
    0 <= value && value <= 255
}

/// returns true if value fits into i32 (signed 32bits).
pub fn fits_i32(value: i64) -> bool {
    i32::MIN as i64 <= value && value <= i32::MAX as i64
}
