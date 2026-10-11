//! 官方按键编码：DirectInput 扫描码（DIK）。
//!
//! 编码来源：`ARAD.exe` 的 `sub_415240`（引擎键盘设备容器初始化），
//! 该函数把「逻辑键槽 → DIK 扫描码」整表写入输入对象 `+0x310`。
//! 逆向源码见 `src/nengine/gfx_texture.cpp`（SECTION 0x415020 键盘输入族）。
//!
//! 注意：DNF 的**游戏内移动/攻击判定走 DirectInput 扫描码**（不是 VK），
//! 因此本项目内部一律用 `Dik` 表达键位；Windows 层用
//! `MapVirtualKeyW(vk, MAPVK_VK_TO_VSC)` 把 `GetAsyncKeyState` 的 VK 转成 DIK。

/// DirectInput 扫描码（0..=0xFF）。
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Dik(pub u8);

impl Dik {
    pub const fn new(code: u8) -> Self {
        Dik(code)
    }
    pub const fn code(self) -> u8 {
        self.0
    }
}

impl core::fmt::Display for Dik {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match dik_name(self.0) {
            Some(n) => write!(f, "{}", n),
            None => write!(f, "DIK({:#04X})", self.0),
        }
    }
}

macro_rules! diks {
    ($($name:ident = $code:expr, $label:expr;)*) => {
        impl Dik {
            $(pub const $name: Dik = Dik($code);)*
        }
        /// 扫描码 → 可读名（用于 HUD / 键位表输出）。
        pub fn dik_name(code: u8) -> Option<&'static str> {
            match code {
                $($code => Some($label),)*
                _ => None,
            }
        }
    };
}

diks! {
    ESCAPE = 0x01, "Esc";
    KEY_1 = 0x02, "1"; KEY_2 = 0x03, "2"; KEY_3 = 0x04, "3";
    KEY_4 = 0x05, "4"; KEY_5 = 0x06, "5"; KEY_6 = 0x07, "6";
    KEY_7 = 0x08, "7"; KEY_8 = 0x09, "8"; KEY_9 = 0x0A, "9";
    KEY_0 = 0x0B, "0";
    MINUS = 0x0C, "-"; EQUALS = 0x0D, "="; BACK = 0x0E, "Back"; TAB = 0x0F, "Tab";
    Q = 0x10, "Q"; W = 0x11, "W"; E = 0x12, "E"; R = 0x13, "R"; T = 0x14, "T";
    Y = 0x15, "Y"; U = 0x16, "U"; I = 0x17, "I"; O = 0x18, "O"; P = 0x19, "P";
    LBRACKET = 0x1A, "["; RBRACKET = 0x1B, "]"; RETURN = 0x1C, "Enter";
    LCONTROL = 0x1D, "LCtrl";
    A = 0x1E, "A"; S = 0x1F, "S"; D = 0x20, "D"; F = 0x21, "F"; G = 0x22, "G"; H = 0x23, "H";
    J = 0x24, "J"; K = 0x25, "K"; L = 0x26, "L";
    SEMICOLON = 0x27, ";"; APOSTROPHE = 0x28, "'"; GRAVE = 0x29, "`";
    LSHIFT = 0x2A, "LShift"; BACKSLASH = 0x2B, "\\";
    Z = 0x2C, "Z"; X = 0x2D, "X"; C = 0x2E, "C"; V = 0x2F, "V";
    B = 0x30, "B"; N = 0x31, "N"; M = 0x32, "M";
    COMMA = 0x33, ","; PERIOD = 0x34, "."; SLASH = 0x35, "/";
    RSHIFT = 0x36, "RShift";
    MULTIPLY = 0x37, "Num*"; LALT = 0x38, "LAlt"; SPACE = 0x39, "Space";
    CAPITAL = 0x3A, "Caps";
    F1 = 0x3B, "F1"; F2 = 0x3C, "F2"; F3 = 0x3D, "F3"; F4 = 0x3E, "F4"; F5 = 0x3F, "F5";
    F6 = 0x40, "F6"; F7 = 0x41, "F7"; F8 = 0x42, "F8"; F9 = 0x43, "F9"; F10 = 0x44, "F10";
    NUMPAD7 = 0x47, "Num7"; NUMPAD8 = 0x48, "Num8"; NUMPAD9 = 0x49, "Num9";
    SUBTRACT = 0x4A, "Num-";
    NUMPAD4 = 0x4B, "Num4"; NUMPAD5 = 0x4C, "Num5"; NUMPAD6 = 0x4D, "Num6";
    ADD = 0x4E, "Num+";
    NUMPAD1 = 0x4F, "Num1"; NUMPAD2 = 0x50, "Num2"; NUMPAD3 = 0x51, "Num3";
    NUMPAD0 = 0x52, "Num0"; DECIMAL = 0x53, "Num.";
    F11 = 0x57, "F11"; F12 = 0x58, "F12";
    RCONTROL = 0x9D, "RCtrl"; NUMPADSLASH = 0xB5, "Num/"; RALT = 0xB8, "RAlt";
    HOME = 0xC7, "Home";
    UP = 0xC8, "Up"; PRIOR = 0xC9, "PgUp";
    LEFT = 0xCB, "Left"; RIGHT = 0xCD, "Right";
    END = 0xCF, "End"; DOWN = 0xD0, "Down"; NEXT = 0xD1, "PgDn";
    INSERT = 0xD2, "Ins"; DELETE = 0xD3, "Del";
}

/// 8 方向输入位掩码中用到的四个方向扫描码。
pub const DIRECTIONS: [Dik; 4] = [Dik::UP, Dik::DOWN, Dik::LEFT, Dik::RIGHT];
