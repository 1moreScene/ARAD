//! Foreground/nonexclusive DirectInput keyboard, matching 4156E0/415770's 256-byte scan-code path.
use std::ffi::c_void;
#[repr(C)]
struct Guid {
    a: u32,
    b: u16,
    c: u16,
    d: [u8; 8],
}
const IID_INPUT8W: Guid = Guid {
    a: 0xbf798031,
    b: 0x483a,
    c: 0x4da2,
    d: [0xaa, 0x99, 0x5d, 0x64, 0xed, 0x36, 0x97, 0],
};
const SYS_KEYBOARD: Guid = Guid {
    a: 0x6f1d2b61,
    b: 0xd5a0,
    c: 0x11cf,
    d: [0xbf, 0xc7, 0x44, 0x45, 0x53, 0x54, 0, 0],
};
#[repr(C)]
struct DataFormat {
    size: u32,
    object_size: u32,
    flags: u32,
    data_size: u32,
    objects: u32,
    formats: *const c_void,
}
// SDK library contains static c_dfDIKeyboard data as well as the DLL function thunk.
#[link(name = "dinput8", kind = "static")]
extern "system" {
    fn DirectInput8Create(
        instance: *mut c_void,
        version: u32,
        iid: *const Guid,
        out: *mut *mut c_void,
        outer: *mut c_void,
    ) -> i32;
    static c_dfDIKeyboard: DataFormat;
}
#[link(name = "dxguid", kind = "static")]
extern "C" {}
#[link(name = "kernel32")]
extern "system" {
    fn GetModuleHandleW(name: *const u16) -> *mut c_void;
}
unsafe fn slot(object: *mut c_void, index: usize) -> *const c_void {
    let table = *(object as *const *const *const c_void);
    *table.add(index)
}
type Simple = unsafe extern "system" fn(*mut c_void) -> i32;
unsafe fn simple(object: *mut c_void, index: usize) -> i32 {
    let call: Simple = std::mem::transmute(slot(object, index));
    call(object)
}
pub struct Keyboard {
    input: *mut c_void,
    device: *mut c_void,
    active: bool,
    samples: u64,
    failures: u64,
}
impl Keyboard {
    pub fn new(hwnd: *mut c_void) -> Result<Self, String> {
        unsafe {
            let mut out = Self {
                input: std::ptr::null_mut(),
                device: std::ptr::null_mut(),
                active: false,
                samples: 0,
                failures: 0,
            };
            let hr = DirectInput8Create(
                GetModuleHandleW(std::ptr::null()),
                0x800,
                &IID_INPUT8W,
                &mut out.input,
                std::ptr::null_mut(),
            );
            if hr < 0 {
                return Err(format!("DirectInput8Create {hr:#x}"));
            }
            type Create = unsafe extern "system" fn(
                *mut c_void,
                *const Guid,
                *mut *mut c_void,
                *mut c_void,
            ) -> i32;
            let create: Create = std::mem::transmute(slot(out.input, 3));
            let hr = create(
                out.input,
                &SYS_KEYBOARD,
                &mut out.device,
                std::ptr::null_mut(),
            );
            if hr < 0 {
                return Err(format!("CreateDevice keyboard {hr:#x}"));
            }
            type Format = unsafe extern "system" fn(*mut c_void, *const DataFormat) -> i32;
            let format: Format = std::mem::transmute(slot(out.device, 11));
            let hr = format(out.device, std::ptr::addr_of!(c_dfDIKeyboard));
            if hr < 0 {
                return Err(format!("SetDataFormat {hr:#x}"));
            }
            type Cooperative = unsafe extern "system" fn(*mut c_void, *mut c_void, u32) -> i32;
            let cooperative: Cooperative = std::mem::transmute(slot(out.device, 13));
            let hr = cooperative(out.device, hwnd, 6); // DISCL_NONEXCLUSIVE | DISCL_FOREGROUND, original4156E0.
            if hr < 0 {
                return Err(format!("SetCooperativeLevel {hr:#x}"));
            }
            println!("键盘后端: DirectInput8 / c_dfDIKeyboard / 256 DIK / FOREGROUND|NONEXCLUSIVE");
            Ok(out)
        }
    }
    pub fn set_active(&mut self, active: bool) {
        if self.active && !active {
            unsafe {
                simple(self.device, 8);
            }
        }
        self.active = active;
    }
    pub fn sample(&mut self) -> Option<Vec<(u8, bool)>> {
        if !self.active {
            return None;
        }
        unsafe {
            if simple(self.device, 7) < 0 {
                self.failures += 1;
                return None;
            } // Acquire, no retry loop.
            simple(self.device, 25); // Poll (keyboard needs no explicit poll, retained for engine parity).
            type State = unsafe extern "system" fn(*mut c_void, u32, *mut c_void) -> i32;
            let state: State = std::mem::transmute(slot(self.device, 9));
            let mut bytes = [0u8; 256];
            if state(self.device, 256, bytes.as_mut_ptr().cast()) < 0 {
                self.failures += 1;
                return None;
            }
            self.samples += 1;
            Some(decode(&bytes))
        }
    }
}
fn decode(bytes: &[u8; 256]) -> Vec<(u8, bool)> {
    bytes
        .iter()
        .enumerate()
        .filter(|(_, byte)| **byte & 0x80 != 0)
        .map(|(scan, _)| (scan as u8, true))
        .collect()
}
impl Drop for Keyboard {
    fn drop(&mut self) {
        unsafe {
            if !self.device.is_null() {
                simple(self.device, 8);
                simple(self.device, 2);
            }
            if !self.input.is_null() {
                simple(self.input, 2);
            }
        }
        if self.samples > 0 {
            println!("keyboard samples={} failed={}", self.samples, self.failures);
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scan_states_preserve_both_enter_keys_and_modifier_sides() {
        let mut bytes = [0; 256];
        for code in [0x1c, 0x9c, 0x2a, 0x36, 0x1d, 0x9d] {
            bytes[code] = 0x80;
        }
        bytes[0x3a] = 1; // A toggle/low-bit flag alone is not a held key.
        let decoded = decode(&bytes);
        assert_eq!(decoded.len(), 6);
        for code in [0x1c, 0x9c, 0x2a, 0x36, 0x1d, 0x9d] {
            assert!(decoded.contains(&(code, true)));
        }
        assert!(!decoded.contains(&(0x3a, true)));
        unsafe {
            assert_eq!(c_dfDIKeyboard.data_size, 256);
            assert_eq!(
                c_dfDIKeyboard.size,
                std::mem::size_of::<DataFormat>() as u32
            );
        }
    }
}
