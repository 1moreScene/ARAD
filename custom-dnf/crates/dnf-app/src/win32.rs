//! Win32/GDI presentation: original 640×480 canvas, explicit bilinear DPI enlargement.
//! DirectInput8 keyboard; GetAsyncKeyState only when device initialization fails.
//! Uncapped rendering with a separate 300Hz simulation clock, IME/focus policies.

// FFI 类型名刻意与 Win32 SDK 保持一致（WNDCLASSW / BITMAPINFO 等），便于对照官方头文件。
#![allow(clippy::upper_case_acronyms)]

use dnf_render::framebuffer::{Framebuffer, HEIGHT, WIDTH};
use std::ffi::c_void;

pub type HWND = *mut c_void;
pub type HINSTANCE = *mut c_void;
pub type HDC = *mut c_void;
pub type HMENU = *mut c_void;
pub type WPARAM = usize;
pub type LPARAM = isize;
pub type LRESULT = isize;
pub type HICON = *mut c_void;
pub type HCURSOR = *mut c_void;
pub type HBRUSH = *mut c_void;
pub type LPCWSTR = *const u16;
pub type UINT = u32;
pub type BOOL = i32;

#[repr(C)]
pub struct WNDCLASSW {
    pub style: UINT,
    pub lpfn_wnd_proc: Option<unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT>,
    pub cb_cls_extra: i32,
    pub cb_wnd_extra: i32,
    pub h_instance: HINSTANCE,
    pub h_icon: HICON,
    pub h_cursor: HCURSOR,
    pub hbr_background: HBRUSH,
    pub lpsz_menu_name: LPCWSTR,
    pub lpsz_class_name: LPCWSTR,
}

#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct POINT {
    pub x: i32,
    pub y: i32,
}

#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct RECT {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

#[repr(C)]
pub struct MSG {
    pub hwnd: HWND,
    pub message: UINT,
    pub w_param: WPARAM,
    pub l_param: LPARAM,
    pub time: u32,
    pub pt: POINT,
    pub private: u32,
}

#[repr(C)]
pub struct BITMAPINFOHEADER {
    pub bi_size: u32,
    pub bi_width: i32,
    pub bi_height: i32,
    pub bi_planes: u16,
    pub bi_bit_count: u16,
    pub bi_compression: u32,
    pub bi_size_image: u32,
    pub bi_x_pels_per_meter: i32,
    pub bi_y_pels_per_meter: i32,
    pub bi_clr_used: u32,
    pub bi_clr_important: u32,
}

#[repr(C)]
pub struct BITMAPINFO {
    pub header: BITMAPINFOHEADER,
    pub colors: [u32; 1],
}

#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct PAINTSTRUCT {
    pub hdc: HDC,
    pub f_erase: BOOL,
    pub rc_paint: RECT,
    pub f_restore: BOOL,
    pub f_inc_update: BOOL,
    pub rgb_reserved: [u8; 32],
}

impl BITMAPINFO {
    fn top_down_rgba(width: i32, height: i32) -> Self {
        BITMAPINFO {
            header: BITMAPINFOHEADER {
                bi_size: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                bi_width: width,
                bi_height: -height, // 负高度 = 自上而下
                bi_planes: 1,
                bi_bit_count: 32,
                bi_compression: BI_RGB,
                bi_size_image: 0,
                bi_x_pels_per_meter: 0,
                bi_y_pels_per_meter: 0,
                bi_clr_used: 0,
                bi_clr_important: 0,
            },
            colors: [0],
        }
    }
}

pub const WM_DESTROY: UINT = 0x0002;
pub const WM_CLOSE: UINT = 0x0010;
pub const WM_QUIT: UINT = 0x0012;
pub const WM_PAINT: UINT = 0x000F;
pub const WM_INPUTLANGCHANGE: UINT = 0x0051;
pub const WM_IME_SETCONTEXT: UINT = 0x0281;
pub const WM_IME_STARTCOMPOSITION: UINT = 0x010D;
pub const WM_IME_ENDCOMPOSITION: UINT = 0x010E;
pub const WM_IME_COMPOSITION: UINT = 0x010F;
pub const WM_ERASEBKGND: UINT = 0x0014;
pub const SW_SHOW: i32 = 5;
pub const WS_OVERLAPPEDWINDOW: u32 = 0x00CF_0000;
pub const WS_FIXED_GAME: u32 = WS_OVERLAPPEDWINDOW & !0x0004_0000 & !0x0001_0000;
pub const WS_VISIBLE: u32 = 0x1000_0000;
pub const CW_USEDEFAULT: i32 = 0x8000_0000u32 as i32;
pub const BI_RGB: u32 = 0;
pub const DIB_RGB_COLORS: UINT = 0;
pub const SRCCOPY: u32 = 0x00CC_0020;
pub const COLORONCOLOR: i32 = 3;
pub const GWLP_USERDATA: i32 = -21;
pub const PM_REMOVE: UINT = 0x0001;
pub const IDC_ARROW: LPCWSTR = 32512 as LPCWSTR;

#[link(name = "user32")]
extern "system" {
    fn RegisterClassW(lpwc: *const WNDCLASSW) -> u16;
    fn CreateWindowExW(
        ex_style: u32,
        class_name: LPCWSTR,
        window_name: LPCWSTR,
        style: u32,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        parent: HWND,
        menu: HMENU,
        instance: HINSTANCE,
        param: *mut c_void,
    ) -> HWND;
    fn DestroyWindow(hwnd: HWND) -> BOOL;
    fn DefWindowProcW(hwnd: HWND, msg: UINT, w: WPARAM, l: LPARAM) -> LRESULT;
    fn ShowWindow(hwnd: HWND, cmd: i32) -> BOOL;
    fn UpdateWindow(hwnd: HWND) -> BOOL;
    fn PeekMessageW(msg: *mut MSG, hwnd: HWND, min: UINT, max: UINT, remove: UINT) -> BOOL;
    fn TranslateMessage(msg: *const MSG) -> BOOL;
    fn DispatchMessageW(msg: *const MSG) -> LRESULT;
    fn PostQuitMessage(code: i32);
    fn GetClientRect(hwnd: HWND, rect: *mut RECT) -> BOOL;
    fn GetCursorPos(point: *mut POINT) -> BOOL;
    fn ScreenToClient(hwnd: HWND, point: *mut POINT) -> BOOL;
    fn GetDC(hwnd: HWND) -> HDC;
    fn ReleaseDC(hwnd: HWND, hdc: HDC) -> i32;
    fn AdjustWindowRectEx(rect: *mut RECT, style: u32, menu: BOOL, ex_style: u32) -> BOOL;
    fn SetWindowLongPtrW(hwnd: HWND, index: i32, value: isize) -> isize;
    fn GetWindowLongPtrW(hwnd: HWND, index: i32) -> isize;
    fn LoadCursorW(instance: HINSTANCE, name: LPCWSTR) -> HCURSOR;
    fn GetAsyncKeyState(vk: i32) -> i16;
    fn MapVirtualKeyW(code: UINT, map_type: UINT) -> UINT;
    fn BeginPaint(hwnd: HWND, ps: *mut PAINTSTRUCT) -> HDC;
    fn EndPaint(hwnd: HWND, ps: *const PAINTSTRUCT) -> BOOL;
    fn SetProcessDPIAware() -> BOOL;
    fn GetDpiForSystem() -> UINT;
    fn GetForegroundWindow() -> HWND;
    fn GetFocus() -> HWND;
    fn IsIconic(hwnd: HWND) -> BOOL;
    fn IsWindow(hwnd: HWND) -> BOOL;
}

#[link(name = "imm32")]
extern "system" {
    fn ImmDisableIME(thread_id: u32) -> BOOL;
    fn ImmAssociateContextEx(hwnd: HWND, context: *mut c_void, flags: u32) -> BOOL;
    fn ImmGetContext(hwnd: HWND) -> *mut c_void;
    fn ImmReleaseContext(hwnd: HWND, context: *mut c_void) -> BOOL;
}

#[link(name = "gdi32")]
extern "system" {
    fn StretchDIBits(
        hdc: HDC,
        x_dst: i32,
        y_dst: i32,
        w_dst: i32,
        h_dst: i32,
        x_src: i32,
        y_src: i32,
        w_src: i32,
        h_src: i32,
        bits: *const c_void,
        info: *const BITMAPINFO,
        usage: UINT,
        rop: u32,
    ) -> i32;
    fn SetStretchBltMode(hdc: HDC, mode: i32) -> i32;
    fn CreateCompatibleDC(hdc: HDC) -> HDC;
    fn DeleteDC(hdc: HDC) -> BOOL;
    fn CreateDIBSection(
        hdc: HDC,
        info: *const BITMAPINFO,
        usage: UINT,
        bits: *mut *mut c_void,
        section: *mut c_void,
        offset: u32,
    ) -> *mut c_void;
    fn SelectObject(hdc: HDC, obj: *mut c_void) -> *mut c_void;
    fn DeleteObject(obj: *mut c_void) -> BOOL;
    fn GdiFlush() -> BOOL;
    fn CreateFontW(
        height: i32,
        width: i32,
        escapement: i32,
        orientation: i32,
        weight: i32,
        italic: u32,
        underline: u32,
        strikeout: u32,
        charset: u32,
        out_precision: u32,
        clip_precision: u32,
        quality: u32,
        pitch: u32,
        face: LPCWSTR,
    ) -> *mut c_void;
    fn SetTextColor(dc: HDC, color: u32) -> u32;
    fn SetBkColor(dc: HDC, color: u32) -> u32;
    fn TextOutW(dc: HDC, x: i32, y: i32, text: LPCWSTR, count: i32) -> BOOL;
    fn GetTextExtentPoint32W(dc: HDC, text: LPCWSTR, count: i32, size: *mut POINT) -> BOOL;
    fn BitBlt(
        hdc_dst: HDC,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        hdc_src: HDC,
        x_src: i32,
        y_src: i32,
        rop: u32,
    ) -> BOOL;
}

#[link(name = "kernel32")]
extern "system" {
    fn GetModuleHandleW(name: LPCWSTR) -> HINSTANCE;
    fn QueryPerformanceCounter(out: *mut i64) -> BOOL;
    fn QueryPerformanceFrequency(out: *mut i64) -> BOOL;
    fn Sleep(ms: u32);
    fn SwitchToThread() -> BOOL;
    fn GetCurrentThreadId() -> u32;
}

pub fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// Logical UI coordinates follow the same physical client/canvas scale as presentation.
pub fn poll_pointer(p: &Presenter) -> (i32, i32, bool) {
    unsafe {
        let mut point = POINT::default();
        let mut rect = RECT::default();
        if GetCursorPos(&mut point) == 0
            || ScreenToClient(p.hwnd, &mut point) == 0
            || GetClientRect(p.hwnd, &mut rect) == 0
        {
            return (-1, -1, false);
        }
        (
            point.x * WIDTH as i32 / (rect.right - rect.left).max(1),
            point.y * HEIGHT as i32 / (rect.bottom - rect.top).max(1),
            key_down(1),
        )
    }
}

/// Unicode UI text -> cached RGBA alpha mask; GDI is only used on a private memory DC.
pub fn raster_text(s: &str) -> Option<(usize, usize, Vec<u8>)> {
    unsafe {
        let dc = CreateCompatibleDC(std::ptr::null_mut());
        if dc.is_null() {
            return None;
        }
        let face = wide("Microsoft YaHei");
        let chars: Vec<u16> = s.encode_utf16().collect();
        let font = CreateFontW(-12, 0, 0, 0, 400, 0, 0, 0, 1, 0, 0, 3, 0, face.as_ptr());
        if font.is_null() {
            DeleteDC(dc);
            return None;
        }
        let old_font = SelectObject(dc, font);
        let mut size = POINT::default();
        GetTextExtentPoint32W(dc, chars.as_ptr(), chars.len() as i32, &mut size);
        let w = size.x.max(1);
        let h = size.y.max(1);
        let info = BITMAPINFO::top_down_rgba(w, h);
        let mut bits = std::ptr::null_mut();
        let bitmap = CreateDIBSection(
            dc,
            &info,
            DIB_RGB_COLORS,
            &mut bits,
            std::ptr::null_mut(),
            0,
        );
        if bitmap.is_null() || bits.is_null() {
            SelectObject(dc, old_font);
            DeleteObject(font);
            DeleteDC(dc);
            return None;
        }
        let old_bitmap = SelectObject(dc, bitmap);
        std::ptr::write_bytes(bits as *mut u8, 0, (w * h * 4) as usize);
        SetTextColor(dc, 0x00ffffff);
        SetBkColor(dc, 0);
        TextOutW(dc, 0, 0, chars.as_ptr(), chars.len() as i32);
        // GDI batches DIB writes: force completion before reading its backing memory.
        GdiFlush();
        let data = std::slice::from_raw_parts(bits as *const u8, (w * h * 4) as usize);
        let rgba = data
            .chunks_exact(4)
            .flat_map(|p| [255, 255, 255, p[0].max(p[1]).max(p[2])])
            .collect();
        SelectObject(dc, old_bitmap);
        SelectObject(dc, old_font);
        DeleteObject(bitmap);
        DeleteObject(font);
        DeleteDC(dc);
        Some((w as usize, h as usize, rgba))
    }
}

pub fn qpc_frequency() -> i64 {
    let mut f = 0i64;
    unsafe {
        QueryPerformanceFrequency(&mut f);
    }
    f.max(1)
}

pub fn qpc() -> i64 {
    let mut c = 0i64;
    unsafe {
        QueryPerformanceCounter(&mut c);
    }
    c
}

pub fn sleep_ms(ms: u32) {
    unsafe { Sleep(ms) }
}

/// RGBA → BGRA（GDI 的 32bpp `BI_RGB` DIB 是 BGRX 字节序）。
pub fn rgba_to_bgra(fb: &Framebuffer, out: &mut [u8]) {
    for (dst, src) in out.chunks_exact_mut(4).zip(fb.data.chunks_exact(4)) {
        dst[0] = src[2];
        dst[1] = src[1];
        dst[2] = src[0];
        dst[3] = 255;
    }
}

/// 帧呈现器：把 RGBA 帧缓冲转成 BGRA 并拉伸到目标 DC。
pub struct Presenter {
    pub performance: crate::performance::Snapshot,
    pub last_render_ms: f64,
    last_present: crate::performance::FrameTime,
    pub hwnd: HWND,
    bgra: Vec<u8>,
    pub width: i32,
    pub height: i32,
    pub input_active: bool,
    pub fps: f64,
    display: std::cell::RefCell<(dnf_render::sampling::LinearScaler, Vec<u8>)>,
    keyboard: Option<crate::keyboard::Keyboard>,
}

impl Presenter {
    pub fn new(hwnd: HWND, width: i32, height: i32) -> Self {
        let keyboard = match crate::keyboard::Keyboard::new(hwnd) {
            Ok(device) => Some(device),
            Err(error) => {
                eprintln!("DirectInput键盘不可用，退回GetAsyncKeyState: {error}");
                None
            }
        };
        Presenter {
            performance: crate::performance::Snapshot::default(),
            last_render_ms: 0.0,
            last_present: crate::performance::FrameTime::default(),
            hwnd,
            bgra: vec![0; (width * height * 4) as usize],
            width,
            height,
            input_active: false,
            fps: 0.0,
            display: std::cell::RefCell::new((
                dnf_render::sampling::LinearScaler::new(
                    (width as usize, height as usize),
                    (width as usize, height as usize),
                ),
                Vec::new(),
            )),
            keyboard,
        }
    }

    pub fn poll_keyboard(&mut self) -> Option<Vec<(u8, bool)>> {
        if !self.input_active {
            return None;
        }
        if let Some(device) = &mut self.keyboard {
            device.sample()
        } else {
            Some(poll_keyboard_fallback())
        }
    }

    /// Observe Windows focus only. Never make the window topmost or reclaim input.
    fn refresh_focus(&mut self) {
        unsafe {
            self.input_active = IsIconic(self.hwnd) == 0
                && GetForegroundWindow() == self.hwnd
                && GetFocus() == self.hwnd;
            if let Some(device) = &mut self.keyboard {
                device.set_active(self.input_active);
            }
        }
    }

    /// 由帧缓冲更新内部 DIB 位图（RGBA → BGRA）。
    pub fn update_from(&mut self, fb: &Framebuffer) {
        rgba_to_bgra(fb, &mut self.bgra);
    }

    /// 把当前 DIB 位图拉伸铺满给定 DC 的客户区（`WM_PAINT` 与主动刷新共用）。
    pub fn blit_to(&self, hdc: HDC) -> crate::performance::FrameTime {
        if hdc.is_null() {
            return crate::performance::FrameTime::default();
        }
        unsafe {
            let mut rc = RECT::default();
            GetClientRect(self.hwnd, &mut rc);
            let cw = (rc.right - rc.left).max(1);
            let ch = (rc.bottom - rc.top).max(1);
            let mut display = self.display.borrow_mut();
            if display.0.dest != (cw as usize, ch as usize) {
                display.0 = dnf_render::sampling::LinearScaler::new(
                    (self.width as usize, self.height as usize),
                    (cw as usize, ch as usize),
                );
            }
            let (scaler, pixels) = &mut *display;
            let start = std::time::Instant::now();
            scaler.render(&self.bgra, pixels);
            let scale_ms = crate::performance::ms(start);
            let info = BITMAPINFO::top_down_rgba(cw, ch);
            let start = std::time::Instant::now();
            // Filtering is done explicitly once; GDI then copies the physical-size DIB 1:1.
            SetStretchBltMode(hdc, COLORONCOLOR);
            StretchDIBits(
                hdc,
                0,
                0,
                cw,
                ch,
                0,
                0,
                cw,
                ch,
                pixels.as_ptr() as *const c_void,
                &info,
                DIB_RGB_COLORS,
                SRCCOPY,
            );
            crate::performance::FrameTime {
                scale: scale_ms,
                gdi: crate::performance::ms(start),
                ..Default::default()
            }
        }
    }

    /// 取窗口客户区 DC 后刷新。
    pub fn blit(&self) -> crate::performance::FrameTime {
        unsafe {
            let hdc = GetDC(self.hwnd);
            if hdc.is_null() {
                return crate::performance::FrameTime::default();
            }
            let times = self.blit_to(hdc);
            ReleaseDC(self.hwnd, hdc);
            times
        }
    }

    /// 更新 + 呈现（等价于 `update_from` + `blit`）。
    pub fn present(&mut self, fb: &Framebuffer) {
        let start = std::time::Instant::now();
        self.update_from(fb);
        let convert = crate::performance::ms(start);
        self.last_present = self.blit();
        self.last_present.convert = convert;
    }
}

/// 把窗口客户区的**真实内容**截回一个 RGBA 帧缓冲（验证"窗口上到底有没有画面"）。
pub fn capture_client(hwnd: HWND, width: i32, height: i32) -> Result<Framebuffer, String> {
    use dnf_render::framebuffer::Framebuffer;
    unsafe {
        let hdc_win = GetDC(hwnd);
        if hdc_win.is_null() {
            return Err("GetDC(hwnd) 失败".into());
        }
        let hdc_mem = CreateCompatibleDC(hdc_win);
        if hdc_mem.is_null() {
            ReleaseDC(hwnd, hdc_win);
            return Err("CreateCompatibleDC 失败".into());
        }
        let info = BITMAPINFO::top_down_rgba(width, height);
        let mut bits: *mut c_void = std::ptr::null_mut();
        let hbmp = CreateDIBSection(
            hdc_mem,
            &info,
            DIB_RGB_COLORS,
            &mut bits,
            std::ptr::null_mut(),
            0,
        );
        if hbmp.is_null() || bits.is_null() {
            DeleteDC(hdc_mem);
            ReleaseDC(hwnd, hdc_win);
            return Err("CreateDIBSection 失败".into());
        }
        let old = SelectObject(hdc_mem, hbmp);
        let ok = BitBlt(hdc_mem, 0, 0, width, height, hdc_win, 0, 0, SRCCOPY);

        let len = (width * height * 4) as usize;
        let bgra = std::slice::from_raw_parts(bits as *const u8, len);
        let mut fb = Framebuffer::new(width as usize, height as usize);
        for (dst, src) in fb.data.chunks_exact_mut(4).zip(bgra.chunks_exact(4)) {
            dst[0] = src[2];
            dst[1] = src[1];
            dst[2] = src[0];
            dst[3] = 255;
        }

        SelectObject(hdc_mem, old);
        DeleteObject(hbmp);
        DeleteDC(hdc_mem);
        ReleaseDC(hwnd, hdc_win);

        if ok == 0 {
            return Err("BitBlt 失败".into());
        }
        Ok(fb)
    }
}

/// GDI 呈现路径自检：在**内存 DC + DIB 段**上跑一遍与窗口完全相同的
/// `StretchDIBits` 调用，然后直接读回位图内容，返回 `(总像素, 非黑像素, 左上角像素)`。
///
/// 这样即使没有可视化环境，也能实证「RGBA→BGRA 转换 + BITMAPINFO 布局 + StretchDIBits 调用」
/// 这条链是通的（把绘图结果读回来验证，而不是"应该没问题"）。
pub fn gdi_blit_self_test(fb: &Framebuffer) -> Result<(u32, u32, [u8; 4]), String> {
    unsafe {
        let hdc_screen = GetDC(std::ptr::null_mut());
        if hdc_screen.is_null() {
            return Err("GetDC(NULL) 失败".into());
        }
        let hdc_mem = CreateCompatibleDC(hdc_screen);
        if hdc_mem.is_null() {
            ReleaseDC(std::ptr::null_mut(), hdc_screen);
            return Err("CreateCompatibleDC 失败".into());
        }
        let info = BITMAPINFO::top_down_rgba(fb.width as i32, fb.height as i32);
        let mut bits: *mut c_void = std::ptr::null_mut();
        let hbmp = CreateDIBSection(
            hdc_mem,
            &info,
            DIB_RGB_COLORS,
            &mut bits,
            std::ptr::null_mut(),
            0,
        );
        if hbmp.is_null() || bits.is_null() {
            DeleteDC(hdc_mem);
            ReleaseDC(std::ptr::null_mut(), hdc_screen);
            return Err("CreateDIBSection 失败".into());
        }
        let old = SelectObject(hdc_mem, hbmp);

        let mut bgra = vec![0u8; fb.data.len()];
        rgba_to_bgra(fb, &mut bgra);
        SetStretchBltMode(hdc_mem, COLORONCOLOR);
        let scan = StretchDIBits(
            hdc_mem,
            0,
            0,
            fb.width as i32,
            fb.height as i32,
            0,
            0,
            fb.width as i32,
            fb.height as i32,
            bgra.as_ptr() as *const c_void,
            &info,
            DIB_RGB_COLORS,
            SRCCOPY,
        );

        let mut non_black = 0u32;
        let total = (fb.width * fb.height) as u32;
        let px = std::slice::from_raw_parts(bits as *const u8, fb.data.len());
        for p in px.chunks_exact(4) {
            if p[0] != 0 || p[1] != 0 || p[2] != 0 {
                non_black += 1;
            }
        }
        let corner = [px[0], px[1], px[2], px[3]];

        SelectObject(hdc_mem, old);
        DeleteObject(hbmp);
        DeleteDC(hdc_mem);
        ReleaseDC(std::ptr::null_mut(), hdc_screen);

        if scan == 0 {
            return Err(format!("StretchDIBits 返回 0（GDI 错误码 {}）", scan));
        }
        Ok((total, non_black, corner))
    }
}

/// DIK「扩展键」判定：这些 VK 的扫描码需要补 `0x80` 位才是 DirectInput 扫描码。
///
/// 依据：「方向键 / Insert..PgDn / 右 Ctrl / 右 Alt / 小键盘除号 / NumLock」在 PC/AT
/// 键盘上以 `E0` 前缀发送，DirectInput 把它们编码为 `扫描码 | 0x80`。
/// 这正是官方键位表里方向键 = `0xCB/0xCD/0xC8/0xD0`（= `0x4B/0x4D/0x48/0x50 | 0x80`）的原因。
/// 主 Enter（`0x1C`）不属于扩展键，`MapVirtualKeyW` 已经返回正确值。
const EXTENDED_VKS: [i32; 14] = [
    0x2D, // Insert
    0x2E, // Delete
    0x24, // Home
    0x23, // End
    0x21, // Page Up
    0x22, // Page Down
    0x25, // Left
    0x26, // Up
    0x27, // Right
    0x28, // Down
    0xA3, // Right Ctrl
    0xA5, // Right Alt
    0x6F, // Numpad /
    0x90, // Num Lock
];

/// VK → DirectInput 扫描码。
pub fn vk_to_dik(vk: i32) -> Option<u8> {
    if vk <= 0 || vk > 0xFE {
        return None;
    }
    let sc = unsafe { MapVirtualKeyW(vk as UINT, 0) } as u8;
    if sc == 0 {
        return None;
    }
    Some(if EXTENDED_VKS.contains(&vk) {
        sc | 0x80
    } else {
        sc
    })
}

/// 采集本帧键盘：返回 `(DIK, 是否按下)` 列表。
fn poll_keyboard_fallback() -> Vec<(u8, bool)> {
    let mut out = Vec::with_capacity(32);
    for vk in 0x08..=0xFE {
        if matches!(vk, 0x10..=0x12) {
            continue;
        } // Generic modifier VKs falsely alias the opposite side.
        let st = unsafe { GetAsyncKeyState(vk) };
        if st != 0 {
            if let Some(dik) = vk_to_dik(vk) {
                out.push((dik, st as u16 & 0x8000 != 0));
            }
        }
    }
    out
}

/// 是否按下了某个 VK（用于 ESC 退出）。
pub fn key_down(vk: i32) -> bool {
    unsafe { GetAsyncKeyState(vk) as u16 & 0x8000 != 0 }
}

fn window_class_name() -> Vec<u16> {
    wide("DnfWhiteboxWindow")
}

/// Uncapped rendering, independent300Hz world updates. Callback flags are(advance,render).
///
/// `screenshot` = `Some((第几帧后, 输出路径))`：到点把**窗口客户区的真实内容**截成 PNG 并退出，
/// 用来验证「窗口上确实有画面」而不是只验证离屏帧缓冲。
pub fn run_window<F>(
    title: &str,
    logical_size: (i32, i32),
    scale: Option<f64>,
    screenshot: Option<(u64, std::path::PathBuf)>,
    mut tick: F,
) -> i32
where
    F: FnMut(&mut Presenter, bool, bool) -> bool,
{
    const CLIENT_W: i32 = WIDTH as i32;
    const CLIENT_H: i32 = HEIGHT as i32;
    assert_eq!(logical_size, (CLIENT_W, CLIENT_H));

    unsafe {
        // Keep OS coordinate APIs in physical units, then explicitly reproduce
        // ARAD's legacy bitmap enlargement in the presentation size.
        SetProcessDPIAware();
        // Match ARAD's DPI-virtualized display size while retaining 640x480 logic.
        let dpi = GetDpiForSystem().max(96);
        let scale = scale.unwrap_or(dpi as f64 / 96.0);
        let display_w = (CLIENT_W as f64 * scale).round() as i32;
        let display_h = (CLIENT_H as f64 * scale).round() as i32;
        // Must happen BEFORE CreateWindow (Microsoft ImmDisableIME contract).
        let ime_disabled = ImmDisableIME(GetCurrentThreadId()) != 0;

        let hinstance = GetModuleHandleW(std::ptr::null());
        let class_name = window_class_name();
        let cursor = LoadCursorW(std::ptr::null_mut(), IDC_ARROW);
        let wc = WNDCLASSW {
            style: 0,
            lpfn_wnd_proc: Some(wnd_proc),
            cb_cls_extra: 0,
            cb_wnd_extra: 0,
            h_instance: hinstance,
            h_icon: std::ptr::null_mut(),
            h_cursor: cursor,
            hbr_background: std::ptr::null_mut(),
            lpsz_menu_name: std::ptr::null(),
            lpsz_class_name: class_name.as_ptr(),
        };
        if RegisterClassW(&wc) == 0 {
            eprintln!("RegisterClassW 失败");
            return 2;
        }

        let mut rc = RECT {
            left: 0,
            top: 0,
            right: display_w,
            bottom: display_h,
        };
        AdjustWindowRectEx(&mut rc, WS_FIXED_GAME, 0, 0);
        let win_w = rc.right - rc.left;
        let win_h = rc.bottom - rc.top;

        let title_w = wide(title);
        let hwnd = CreateWindowExW(
            0,
            class_name.as_ptr(),
            title_w.as_ptr(),
            WS_FIXED_GAME | WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            win_w,
            win_h,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            hinstance,
            std::ptr::null_mut(),
        );
        if hwnd.is_null() {
            eprintln!("CreateWindowExW 失败");
            return 2;
        }
        ImmAssociateContextEx(hwnd, std::ptr::null_mut(), 0);
        ImmAssociateContextEx(hwnd, std::ptr::null_mut(), 1); // existing children

        let mut presenter = Box::new(Presenter::new(hwnd, CLIENT_W, CLIENT_H));
        SetWindowLongPtrW(
            hwnd,
            GWLP_USERDATA,
            (&mut *presenter as *mut Presenter) as isize,
        );

        // 先把**第一帧**画好再显示窗口，避免首个 WM_PAINT 画出一张空（全黑）位图。
        // `false` = 只渲染不推进逻辑。
        if !tick(&mut presenter, false, true) {
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            DestroyWindow(hwnd);
            return 0;
        }
        ShowWindow(hwnd, SW_SHOW);
        UpdateWindow(hwnd);
        presenter.refresh_focus();
        let mut actual = RECT::default();
        GetClientRect(hwnd, &mut actual);
        let context = ImmGetContext(hwnd);
        println!(
            "窗口策略: 客户区={}x{} IME线程禁用={} IME上下文={} 自由焦点=true 输入焦点={}",
            actual.right - actual.left,
            actual.bottom - actual.top,
            ime_disabled,
            !context.is_null(),
            presenter.input_active
        );
        if !context.is_null() {
            ImmReleaseContext(hwnd, context);
        }
        println!(
            "显示策略: 逻辑{}x{} DPI={} 倍率={:.2} 显示{}x{}",
            CLIENT_W, CLIENT_H, dpi, scale, display_w, display_h
        );
        println!("采样策略: 精灵缩放/旋转LINEAR；DPI显示BILINEAR；16位素材颜色完整位扩展。");

        let freq = qpc_frequency();
        let mut clock = dnf_core::clock::FixedClock::new(dnf_core::clock::WORLD_MS);
        let started = qpc();
        let mut previous = started;
        let mut fps_started = started;
        let mut fps_frames = 0u64;
        let mut rendered = 0u64;
        let mut performance = crate::performance::Monitor::default();
        let mut last_presented = started;
        println!("帧率策略: 渲染无上限；世界逻辑300Hz；FPS为窗口绘制/提交次数，区别于显示器Hz。");

        let mut msg = MSG {
            hwnd: std::ptr::null_mut(),
            message: 0,
            w_param: 0,
            l_param: 0,
            time: 0,
            pt: POINT::default(),
            private: 0,
        };
        let mut frames: u64 = 0;
        let mut running = true;
        while running {
            // 泵消息（**有限**：WM_PAINT 现在会被 BeginPaint/EndPaint 清掉无效区域，
            // 所以这里不会死循环；但加个上限更保险）
            let mut pumped = 0;
            while PeekMessageW(&mut msg, std::ptr::null_mut(), 0, 0, PM_REMOVE) != 0 {
                pumped += 1;
                if pumped > 64 {
                    break;
                }
                if msg.message == WM_QUIT || msg.message == WM_DESTROY {
                    running = false;
                    break;
                }
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
            if !running {
                break;
            }

            let now = qpc();
            let mut logic_ms = 0.0;
            let steps = clock.advance((now - previous) as f64 * 1000.0 / freq as f64);
            previous = now;
            if steps > 0 {
                presenter.refresh_focus();
                for _ in 0..steps {
                    let logic_start = std::time::Instant::now();
                    if !tick(&mut presenter, true, false) {
                        running = false;
                        break;
                    }
                    logic_ms += crate::performance::ms(logic_start);
                    frames += 1;
                    // Screenshot parameters retain the legacy60Hz reference timebase:
                    // shot-after240 =4seconds, independent of world/render frequency.
                    if screenshot.as_ref().is_some_and(|(after, _)| {
                        frames as f64 * dnf_core::clock::WORLD_MS + 1e-7
                            >= *after as f64 * dnf_core::clock::LOGIC_MS
                    }) {
                        break;
                    }
                }
            }
            if !running {
                break;
            }
            if now - fps_started >= freq / 4 {
                presenter.fps = fps_frames as f64 * freq as f64 / (now - fps_started) as f64;
                presenter.performance = performance.snapshot();
                fps_frames = 0;
                fps_started = now;
            }
            if IsIconic(hwnd) == 0 {
                if !tick(&mut presenter, false, true) {
                    break;
                }
                rendered += 1;
                fps_frames += 1;
                let end = qpc();
                performance.push(crate::performance::FrameTime {
                    frame: (end - last_presented) as f64 * 1000.0 / freq as f64,
                    logic: logic_ms,
                    render: presenter.last_render_ms,
                    ..presenter.last_present
                });
                last_presented = end;
            }
            if let Some((after, path)) = screenshot.as_ref() {
                if frames as f64 * dnf_core::clock::WORLD_MS + 1e-7
                    >= *after as f64 * dnf_core::clock::LOGIC_MS
                {
                    match capture_client(hwnd, display_w, display_h) {
                        Ok(fb) => {
                            let non_black = fb
                                .data
                                .chunks_exact(4)
                                .filter(|p| p[0] != 0 || p[1] != 0 || p[2] != 0)
                                .count();
                            match dnf_render::png::write_file(path, &fb) {
                                Ok(()) => println!(
                                    "窗口实拍已保存：{}（{}x{}，非黑像素 {}）",
                                    path.display(),
                                    fb.width,
                                    fb.height,
                                    non_black
                                ),
                                Err(e) => eprintln!("PNG 写出失败：{}", e),
                            }
                        }
                        Err(e) => eprintln!("窗口实拍失败：{}", e),
                    }
                    break;
                }
            }
            if IsIconic(hwnd) != 0 {
                // There is no drawable client area while minimized; logic/audio keep running.
                sleep_ms(1);
            } else {
                // Yield to another ready thread without a timed wait or render-rate ceiling.
                SwitchToThread();
            }
        }
        let elapsed = (qpc() - started) as f64 / freq as f64;
        println!("呈现统计: logic_steps={frames} rendered={rendered} elapsed_s={elapsed:.3} average_fps={:.1} uncapped=true",rendered as f64/elapsed.max(0.001));
        let p = performance.snapshot();
        println!("PERF samples={} mean_frame_ms={:.3} p95_frame_ms={:.3} p99_frame_ms={:.3} slow_frames={} logic_ms={:.3} render_ms={:.3} convert_ms={:.3} scale_ms={:.3} gdi_ms={:.3}",p.samples,p.avg.frame,p.p95,p.p99,p.slow_frames,p.avg.logic,p.avg.render,p.avg.convert,p.avg.scale,p.avg.gdi);
        // Detach userdata before freeing Presenter.
        if IsWindow(hwnd) != 0 {
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            DestroyWindow(hwnd);
        }
    }
    0
}

unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: UINT, w: WPARAM, l: LPARAM) -> LRESULT {
    match msg {
        WM_INPUTLANGCHANGE => {
            ImmAssociateContextEx(hwnd, std::ptr::null_mut(), 0);
            0
        }
        WM_IME_SETCONTEXT
        | WM_IME_STARTCOMPOSITION
        | WM_IME_ENDCOMPOSITION
        | WM_IME_COMPOSITION => 0,
        WM_CLOSE => {
            DestroyWindow(hwnd);
            0
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            0
        }
        WM_ERASEBKGND => 1, // 交给 present 全屏覆盖，避免闪烁
        WM_PAINT => {
            // ⚠️ 必须 BeginPaint/EndPaint：否则无效区域永不清除，Windows 会无限重发
            // WM_PAINT，消息泵被占死（表现为窗口一直全黑、逻辑一帧都不推进）。
            let mut ps = PAINTSTRUCT::default();
            let hdc = BeginPaint(hwnd, &mut ps);
            if !hdc.is_null() {
                let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const Presenter;
                if !ptr.is_null() {
                    (*ptr).blit_to(hdc);
                }
            }
            EndPaint(hwnd, &ps);
            0
        }
        _ => DefWindowProcW(hwnd, msg, w, l),
    }
}
