use crate::mem::*;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct AssemblerLabel {
    pub offset: u32,
}

impl Default for AssemblerLabel {
    fn default() -> Self {
        Self {
            offset: u32::max_value(),
        }
    }
}

impl AssemblerLabel {
    pub const fn new(x: u32) -> Self {
        Self { offset: x }
    }

    pub const fn is_set(&self) -> bool {
        self.offset != u32::max_value()
    }

    pub const fn label_at_offset(&self, offset: u32) -> Self {
        Self::new(self.offset + offset)
    }
}

pub struct AssemblerBuffer {
    pub(crate) storage: Vec<u8>,
    pub(crate) index: usize,
}

impl AssemblerBuffer {
    pub const INLINE_CAPACITY: usize = 128;
}

impl AssemblerBuffer {
    pub fn append(&mut self, data: &[u8]) {
        self.storage.extend(data);
        self.index += data.len();
    }

    pub fn label(&self) -> AssemblerLabel {
        AssemblerLabel::new(self.index as _)
    }

    pub fn code_size(&self) -> usize {
        self.index
    }

    pub fn data(&self) -> &[u8] {
        &self.storage
    }

    pub fn data_mut(&mut self) -> &mut [u8] {
        &mut self.storage
    }

    pub fn executable_memory(&self) -> Option<*mut u8> {
        if self.index == 0 {
            return None;
        }
        let result = commit(align_usize(self.index, page_size()), true);
        if result.is_null() {
            return None;
        }
        unsafe {
            std::ptr::copy_nonoverlapping(self.storage.as_ptr(), result, self.index);
        }
        protect(result, self.index, Access::ReadExecutable);
        Some(result)
    }
    pub fn executable_writable_memory(&self) -> Option<*mut u8> {
        if self.index == 0 {
            return None;
        }
        let result = commit(align_usize(self.index, page_size()), true);
        if result.is_null() {
            return None;
        }
        unsafe {
            std::ptr::copy_nonoverlapping(self.storage.as_ptr(), result, self.index);
        }
        Some(result)
    }
    // https://github.com/rust-lang/rust/issues/69228
    /*pub fn put_integral<T: 'static + Sized + Copy + Clone>(&mut self, x: T) {
        let bytes: [u8; std::mem::size_of::<T>()] = unsafe { std::mem::transmute(x) };
    }*/
    pub fn put_byte(&mut self, value: u8) {
        self.append(&[value as u8]);
    }
    pub fn put_short(&mut self, value: u16) {
        self.append(&unsafe { std::mem::transmute::<u16, [u8; 2]>(value as u16) });
    }

    pub fn put_int(&mut self, value: i32) {
        self.append(&unsafe { std::mem::transmute::<u32, [u8; 4]>(value as u32) });
    }

    pub fn put_long(&mut self, value: u64) {
        self.append(&unsafe { std::mem::transmute::<u64, [u8; 8]>(value as u64) });
    }
}
