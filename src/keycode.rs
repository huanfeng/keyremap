//! 键名 <-> 虚拟键码映射表。
//!
//! 这张表是主程序与 Web 配置工具的共同契约: `--dump-keys` 会把它输出成 JSON,
//! 前端直接消费, 避免两边各维护一份导致命名漂移。
//!
//! 命名原则: 用户看得懂优先, 不暴露 Win32 的 OEM_1 这类内部名。

use windows_sys::Win32::UI::Input::KeyboardAndMouse::*;

/// 输入源: 键盘按键或鼠标按键。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Input {
    Key(u16),
    Mouse(MouseButton),
}

/// 鼠标按键。X1/X2 是侧键 (通常 X1=后退, X2=前进)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    X1,
    X2,
}

/// 鼠标按键名表。
const MOUSE_TABLE: &[(&str, MouseButton)] = &[
    ("MouseLeft", MouseButton::Left),
    ("MouseRight", MouseButton::Right),
    ("MouseMiddle", MouseButton::Middle),
    ("MouseX1", MouseButton::X1),
    ("MouseX2", MouseButton::X2),
];

/// 键盘按键名表。顺序即 `--dump-keys` 的输出顺序, 按功能分组便于前端分类展示。
#[rustfmt::skip]
pub const KEY_TABLE: &[(&str, u16)] = &[
    // 字母
    ("A", VK_A), ("B", VK_B), ("C", VK_C), ("D", VK_D), ("E", VK_E),
    ("F", VK_F), ("G", VK_G), ("H", VK_H), ("I", VK_I), ("J", VK_J),
    ("K", VK_K), ("L", VK_L), ("M", VK_M), ("N", VK_N), ("O", VK_O),
    ("P", VK_P), ("Q", VK_Q), ("R", VK_R), ("S", VK_S), ("T", VK_T),
    ("U", VK_U), ("V", VK_V), ("W", VK_W), ("X", VK_X), ("Y", VK_Y),
    ("Z", VK_Z),

    // 主键盘数字
    ("0", VK_0), ("1", VK_1), ("2", VK_2), ("3", VK_3), ("4", VK_4),
    ("5", VK_5), ("6", VK_6), ("7", VK_7), ("8", VK_8), ("9", VK_9),

    // 功能键
    ("F1", VK_F1), ("F2", VK_F2), ("F3", VK_F3), ("F4", VK_F4),
    ("F5", VK_F5), ("F6", VK_F6), ("F7", VK_F7), ("F8", VK_F8),
    ("F9", VK_F9), ("F10", VK_F10), ("F11", VK_F11), ("F12", VK_F12),
    ("F13", VK_F13), ("F14", VK_F14), ("F15", VK_F15), ("F16", VK_F16),
    ("F17", VK_F17), ("F18", VK_F18), ("F19", VK_F19), ("F20", VK_F20),
    ("F21", VK_F21), ("F22", VK_F22), ("F23", VK_F23), ("F24", VK_F24),

    // 修饰键。不带 L/R 前缀的是"任意侧", 注入时用左键。
    ("Ctrl", VK_CONTROL), ("LCtrl", VK_LCONTROL), ("RCtrl", VK_RCONTROL),
    ("Alt", VK_MENU), ("LAlt", VK_LMENU), ("RAlt", VK_RMENU),
    ("Shift", VK_SHIFT), ("LShift", VK_LSHIFT), ("RShift", VK_RSHIFT),
    ("LWin", VK_LWIN), ("RWin", VK_RWIN),

    // 编辑与导航
    ("Esc", VK_ESCAPE), ("Tab", VK_TAB), ("CapsLock", VK_CAPITAL),
    ("Space", VK_SPACE), ("Enter", VK_RETURN), ("Backspace", VK_BACK),
    ("Insert", VK_INSERT), ("Delete", VK_DELETE),
    ("Home", VK_HOME), ("End", VK_END),
    ("PageUp", VK_PRIOR), ("PageDown", VK_NEXT),
    ("Left", VK_LEFT), ("Right", VK_RIGHT), ("Up", VK_UP), ("Down", VK_DOWN),
    ("PrintScreen", VK_SNAPSHOT), ("ScrollLock", VK_SCROLL), ("Pause", VK_PAUSE),
    ("Apps", VK_APPS), ("NumLock", VK_NUMLOCK),

    // 符号键 (US 布局下的刻印)
    ("Backquote", VK_OEM_3), ("Minus", VK_OEM_MINUS), ("Equal", VK_OEM_PLUS),
    ("LeftBracket", VK_OEM_4), ("RightBracket", VK_OEM_6), ("Backslash", VK_OEM_5),
    ("Semicolon", VK_OEM_1), ("Quote", VK_OEM_7),
    ("Comma", VK_OEM_COMMA), ("Period", VK_OEM_PERIOD), ("Slash", VK_OEM_2),

    // 小键盘
    ("Numpad0", VK_NUMPAD0), ("Numpad1", VK_NUMPAD1), ("Numpad2", VK_NUMPAD2),
    ("Numpad3", VK_NUMPAD3), ("Numpad4", VK_NUMPAD4), ("Numpad5", VK_NUMPAD5),
    ("Numpad6", VK_NUMPAD6), ("Numpad7", VK_NUMPAD7), ("Numpad8", VK_NUMPAD8),
    ("Numpad9", VK_NUMPAD9),
    ("NumpadAdd", VK_ADD), ("NumpadSubtract", VK_SUBTRACT),
    ("NumpadMultiply", VK_MULTIPLY), ("NumpadDivide", VK_DIVIDE),
    ("NumpadDecimal", VK_DECIMAL),

    // 媒体键
    ("VolumeMute", VK_VOLUME_MUTE), ("VolumeDown", VK_VOLUME_DOWN),
    ("VolumeUp", VK_VOLUME_UP),
    ("MediaNext", VK_MEDIA_NEXT_TRACK), ("MediaPrev", VK_MEDIA_PREV_TRACK),
    ("MediaStop", VK_MEDIA_STOP), ("MediaPlayPause", VK_MEDIA_PLAY_PAUSE),
];

/// 按名字解析输入源, 键盘和鼠标统一入口。配置加载时调用, 不在热路径上。
pub fn input_from_name(name: &str) -> Option<Input> {
    if let Some((_, btn)) = MOUSE_TABLE
        .iter()
        .find(|(n, _)| n.eq_ignore_ascii_case(name))
    {
        return Some(Input::Mouse(*btn));
    }
    vk_from_name(name).map(Input::Key)
}

/// 按名字解析虚拟键码。
///
/// 除了 KEY_TABLE 中预定义的键名, 还支持 `VK=0xNN` 格式,
/// 例如 `VK=0xB5`。键名和 VK 前缀均不区分大小写。
pub fn vk_from_name(name: &str) -> Option<u16> {
    // 先查预定义键名, 例如 A、F1、MediaPlayPause 等。
    if let Some(vk) = KEY_TABLE
        .iter()
        .find(|(n, _)| n.eq_ignore_ascii_case(name))
        .map(|(_, vk)| *vk)
    {
        return Some(vk);
    }

    // 再支持 VK=0xNN / vk=0xnn 格式。
    let (prefix, value) = name.split_once('=')?;
    if !prefix.eq_ignore_ascii_case("VK") {
        return None;
    }

    let value = value.trim();
    let value = value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"))?;

    // KBDLLHOOKSTRUCT::vkCode 的有效范围是 0..=255。
    u16::from_str_radix(value, 16)
        .ok()
        .filter(|vk| *vk <= 0xFF)
}

/// 反查键名, 用于 --listen 模式和日志。表只有百余项, 线性扫描足够。
pub fn name_from_vk(vk: u16) -> Option<&'static str> {
    KEY_TABLE.iter().find(|(_, v)| *v == vk).map(|(n, _)| *n)
}

/// 反查鼠标按键名。
pub fn name_from_mouse(btn: MouseButton) -> &'static str {
    MOUSE_TABLE
        .iter()
        .find(|(_, b)| *b == btn)
        .map(|(n, _)| *n)
        .unwrap_or("Mouse?")
}

/// 扩展键判定。
///
/// 这类键在 PS/2 扫描码里带 0xE0 前缀, 注入时必须带 `KEYEVENTF_EXTENDEDKEY`,
/// 否则会被识别成小键盘上的同码位键 —— 典型症状是"映射到 Insert 却变成了数字 0"。
/// 这是自己实现注入时最容易漏掉的一处。
pub fn is_extended_key(vk: u16) -> bool {
    matches!(
        vk,
        VK_RCONTROL
            | VK_RMENU
            | VK_INSERT
            | VK_DELETE
            | VK_HOME
            | VK_END
            | VK_PRIOR
            | VK_NEXT
            | VK_LEFT
            | VK_RIGHT
            | VK_UP
            | VK_DOWN
            | VK_NUMLOCK
            | VK_DIVIDE
            | VK_SNAPSHOT
            | VK_LWIN
            | VK_RWIN
            | VK_APPS
    )
}

/// 把"任意侧"修饰键规范化成具体的左键, 供注入使用。
///
/// `VK_CONTROL`/`VK_MENU`/`VK_SHIFT` 是逻辑键, 直接 SendInput 行为不确定,
/// 必须落成 `VK_LCONTROL` 这类物理键。
pub fn normalize_for_inject(vk: u16) -> u16 {
    match vk {
        VK_CONTROL => VK_LCONTROL,
        VK_MENU => VK_LMENU,
        VK_SHIFT => VK_LSHIFT,
        other => other,
    }
}

// ---------- 组合键输入源的修饰键 ----------

/// `from` 里可以要求按住的修饰键, 按位存放。
///
/// 低 4 位是**不分左右**的通用写法 (`Ctrl` = 左右任一按下即算), 高 8 位则指定
/// 具体某一侧 (`LCtrl` 只认左边那个)。通用写法是常见需求, 分侧写法留给
/// "右 Alt 当第二功能键"这类布局。
pub const MOD_CTRL: u16 = 1 << 0;
pub const MOD_SHIFT: u16 = 1 << 1;
pub const MOD_ALT: u16 = 1 << 2;
pub const MOD_WIN: u16 = 1 << 3;
pub const MOD_LCTRL: u16 = 1 << 4;
pub const MOD_RCTRL: u16 = 1 << 5;
pub const MOD_LSHIFT: u16 = 1 << 6;
pub const MOD_RSHIFT: u16 = 1 << 7;
pub const MOD_LALT: u16 = 1 << 8;
pub const MOD_RALT: u16 = 1 << 9;
pub const MOD_LWIN: u16 = 1 << 10;
pub const MOD_RWIN: u16 = 1 << 11;

/// CapsLock 当修饰键用。
///
/// 它和上面那些不一样: CapsLock 是个普通按键, 按一下就会切换大小写状态。
/// 所以一旦有映射用到它, 钩子必须把它的按下事件吞掉, 见 `hook::handle_caps`。
pub const MOD_CAPS: u16 = 1 << 12;

/// 修饰键名 -> 位。写法与键名表一致地大小写不敏感。
/// 分侧的名字与 [`KEY_TABLE`] 里的一致, 用户不用记两套。
const MOD_TABLE: &[(&str, u16)] = &[
    ("Ctrl", MOD_CTRL),
    ("Control", MOD_CTRL),
    ("Shift", MOD_SHIFT),
    ("Alt", MOD_ALT),
    ("Win", MOD_WIN),
    ("LCtrl", MOD_LCTRL),
    ("RCtrl", MOD_RCTRL),
    ("LShift", MOD_LSHIFT),
    ("RShift", MOD_RSHIFT),
    ("LAlt", MOD_LALT),
    ("RAlt", MOD_RALT),
    ("LWin", MOD_LWIN),
    ("RWin", MOD_RWIN),
    ("CapsLock", MOD_CAPS),
];

/// 每个位对应的 (左键, 右键)。通用位两侧都要看, 分侧位只看自己那一侧。
///
/// 表里第二项为 `None` 表示这个位只认一侧。
const MOD_SIDES: &[(u16, u16, Option<u16>)] = &[
    (MOD_CTRL, VK_LCONTROL, Some(VK_RCONTROL)),
    (MOD_SHIFT, VK_LSHIFT, Some(VK_RSHIFT)),
    (MOD_ALT, VK_LMENU, Some(VK_RMENU)),
    (MOD_WIN, VK_LWIN, Some(VK_RWIN)),
    (MOD_LCTRL, VK_LCONTROL, None),
    (MOD_RCTRL, VK_RCONTROL, None),
    (MOD_LSHIFT, VK_LSHIFT, None),
    (MOD_RSHIFT, VK_RSHIFT, None),
    (MOD_LALT, VK_LMENU, None),
    (MOD_RALT, VK_RMENU, None),
    (MOD_LWIN, VK_LWIN, None),
    (MOD_RWIN, VK_RWIN, None),
    // 这里**没有** MOD_CAPS, 是有意的。CapsLock 的按下事件被钩子吞掉了,
    // 而被吞掉的事件不会进入系统的按键状态 —— 去问 GetAsyncKeyState 永远得到
    // "没按下"。它的按住状态只能由 hook 自己记录, 见 `hook::CAPS_DOWN`。
];

/// 按名字解析修饰键位。不是修饰键就返回 None。
pub fn mod_from_name(name: &str) -> Option<u16> {
    MOD_TABLE
        .iter()
        .find(|(n, _)| n.eq_ignore_ascii_case(name))
        .map(|(_, m)| *m)
}

/// 位掩码的显示名, 例如 `Ctrl + Shift`。空掩码返回空串。
pub fn mods_name(mods: u16) -> String {
    // 固定顺序输出, 与用户在配置里怎么写无关 —— 菜单和日志里同一组修饰键
    // 始终长一个样, 才好一眼比对。
    const ORDER: &[(u16, &str)] = &[
        (MOD_CTRL, "Ctrl"),
        (MOD_LCTRL, "LCtrl"),
        (MOD_RCTRL, "RCtrl"),
        (MOD_SHIFT, "Shift"),
        (MOD_LSHIFT, "LShift"),
        (MOD_RSHIFT, "RShift"),
        (MOD_ALT, "Alt"),
        (MOD_LALT, "LAlt"),
        (MOD_RALT, "RAlt"),
        (MOD_WIN, "Win"),
        (MOD_LWIN, "LWin"),
        (MOD_RWIN, "RWin"),
        (MOD_CAPS, "CapsLock"),
    ];
    ORDER
        .iter()
        .filter(|(bit, _)| mods & bit != 0)
        .map(|(_, n)| *n)
        .collect::<Vec<_>>()
        .join(" + ")
}

/// 某个虚拟键此刻是否按着。最高位表示按下状态。
fn key_down(vk: u16) -> bool {
    unsafe { GetAsyncKeyState(vk as i32) as u16 & 0x8000 != 0 }
}

/// 当前是否按住了 `required` 里要求的**全部**修饰键。空掩码恒为真。
///
/// 用 `GetAsyncKeyState` 现查, 而不是自己维护一份按下状态。自己记会失步 ——
/// 钩子装上之前就按住的键、被别的钩子吞掉的抬起事件, 都会让状态卡在"按着",
/// 而失步的表现是"修饰键像是一直按着", 属于最难排查的那一类问题。
///
/// 这个函数在钩子回调里调用, 但**只有触发键先匹配上了才会走到这里**,
/// 所以一次按键最多几次系统调用, 不构成热路径负担。
pub fn mods_held(required: u16) -> bool {
    if required == 0 {
        return true;
    }
    MOD_SIDES.iter().all(|(bit, left, right)| {
        required & bit == 0 || key_down(*left) || right.is_some_and(key_down)
    })
}

/// 释放修饰键时最多要动的键数: 四组修饰键各有左右两侧。
pub const MAX_MODS: usize = 8;

/// 列出为满足 `required` 而**此刻真正按着**的那些修饰键, 返回 (数组, 有效长度)。
///
/// 注意返回的是"实际按着的键", 而不是"位掩码对应的键"。这个区别是本函数存在的
/// 全部理由:
///
/// 以前这里固定返回左侧键 (`VK_LCONTROL` 之类)。用户按的若是**右** Ctrl,
/// 注入 `LCtrl↑` 毫无作用 —— `VK_CONTROL` 的状态由"左右任一按下"决定, 右边还按着,
/// 应用照样收到 `Ctrl+Backspace`。更糟的是收尾时那一下 `LCtrl↓`: 它按下了一个
/// 用户根本没按的键, 等用户松开右 Ctrl 之后, 左 Ctrl 在系统里就一直是按下状态,
/// 表现为 **Ctrl 卡住**。
///
/// 只要按"当前实际按着的键"来松开、再把同样这几个按回去, 两个问题一起消失。
///
/// 回填到定长数组而不是 `Vec`: 这个函数在钩子回调里被调用, 那里不能有堆分配。
pub fn held_mod_vks(required: u16) -> ([u16; MAX_MODS], usize) {
    let mut out = [0u16; MAX_MODS];
    let mut n = 0;
    let push = |vk: u16, out: &mut [u16; MAX_MODS], n: &mut usize| {
        // 通用位与分侧位可能指向同一个键 (比如同时写了 Ctrl 和 LCtrl), 去个重
        if !out[..*n].contains(&vk) && *n < MAX_MODS {
            out[*n] = vk;
            *n += 1;
        }
    };

    for (bit, left, right) in MOD_SIDES {
        if required & bit == 0 {
            continue;
        }
        if key_down(*left) {
            push(*left, &mut out, &mut n);
        }
        if let Some(r) = right
            && key_down(*r)
        {
            push(*r, &mut out, &mut n);
        }
    }
    (out, n)
}
