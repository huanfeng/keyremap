//! 开机自启动。
//!
//! 有两种机制, 用哪个取决于"是否以管理员身份运行"这个偏好:
//!
//! - **普通权限**: `HKCU\...\Run` 项。简单, 不需要任何特权。
//! - **管理员**:   任务计划程序里一个 `RunLevel=HighestAvailable` 的登录触发任务。
//!
//! 为什么管理员模式必须换机制: `HKCU\Run` 启动的进程**永远不会提权**,
//! 这是 Windows 的设计。想要"开机自启 + 管理员"只有计划任务这一条路。
//!
//! 两种机制互斥, 切换时必须清掉另一个 —— 否则会有两条自启路径同时生效。
//! (虽然单实例锁会挡掉第二个进程, 但那是靠巧合而不是设计。)
//!
//! # 为什么任务用 XML 而不是 schtasks 的命令行参数
//!
//! `schtasks /Create /SC ONLOGON` 会套用一组对常驻程序**致命**的默认值,
//! 而命令行参数没有任何办法改掉它们:
//!
//! - `DisallowStartIfOnBatteries=true` —— 笔记本拔着电源开机, 任务**根本不触发**,
//!   无日志无提示, 表现为"开机自启时灵时不灵"。
//! - `StopIfGoingOnBatteries=true` —— 跑着跑着拔掉电源, 进程被任务计划程序杀掉。
//! - `ExecutionTimeLimit` 省略即取默认的 `PT72H` —— 常驻进程跑满 3 天被强制结束。
//!
//! 所以任务定义走 `/XML`: 那是唯一能把这三项按住的途径。顺带还解决了两件事 ——
//! 命令与参数分别落进 `<Command>`/`<Arguments>` (命令行方式会把整串塞进
//! `<Command>`), 以及登录触发器带上 `<UserId>` (否则语义是"任何用户登录时")。

use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use log::{debug, info, warn};
use windows_sys::Win32::System::Registry::HKEY_CURRENT_USER;

use crate::{elevate, paths, regutil, singleton};

const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const RUN_VALUE: &str = "keyremap";

/// 不让 schtasks 闪出一个控制台窗口 —— 本程序是 GUI 子系统的。
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// `schtasks /Query` 在任务不存在时的退出码。用它把"确实没有"和
/// "查询本身失败"分开, 后者不该被当成"没开自启"。
const SCHTASKS_NOT_FOUND: i32 = 1;

/// 任务定义的格式版本, 建成时记在注册表里。
///
/// 光看"任务存在"是不够的: 老版本用 `schtasks /Create /SC ONLOGON` 建的任务
/// 带着那组致命默认值 (见模块头部), 升级上来之后它照样在那儿, 而
/// [`sync`] 的"机制是否一致"判断认为一切正常 —— 用户升级了却什么都没变好。
/// 有了版本号, [`sync`] 才知道该把旧任务重建一遍。
///
/// **改动 [`task_xml`] 里任何影响行为的设置时, 把这个数加一。**
/// 1 = 0.5.1 及之前的 schtasks 命令行方式; 2 = XML 方式; 3 = 登录触发器加延迟。
const TASK_FORMAT_VERSION: u32 = 3;
const VALUE_TASK_VERSION: &str = "TaskFormatVersion";

/// 计划任务名。带上 exe 路径的哈希, 这样复制到别处的另一份副本
/// 可以有自己的自启任务, 和单实例锁的粒度保持一致。
fn task_name() -> String {
    singleton::name_for_current_exe()
}

/// [`enable`] 的结果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Enabled {
    /// 已经按当前偏好落实到位, 不需要调用方再做什么。
    Done,
    /// 偏好是"以管理员运行"但当前进程没有权限建计划任务。
    /// 已经写了 `HKCU\Run` 项**保底**, 调用方应当提权重启,
    /// 重启后 [`sync`] 会把它换成计划任务。
    NeedsElevation,
}

/// 当前是否已设置开机自启 (两种机制任一生效即算)。
pub fn is_enabled() -> bool {
    run_entry_is_self() || task_exists()
}

/// 按当前的管理员偏好启用自启。
///
/// 会顺手清掉另一种机制, 所以也可以当"切换机制"用。
///
/// 权限不够时**不报错**, 而是退回 Run 项并返回 [`Enabled::NeedsElevation`]:
/// 直接失败的话用户点完菜单只看到一个几秒就消失的气泡, 而实际上一条自启
/// 都没有 —— 这正是"设了自启但重启后没起来"的由来。
pub fn enable(config_path: &Path) -> Result<Enabled, String> {
    if elevate::wants_admin() {
        if !elevate::is_elevated() {
            debug!("暂无权限创建管理员级别的自启任务, 先用 Run 项保底");
            write_run_entry(config_path)?;
            return Ok(Enabled::NeedsElevation);
        }
        // 顺序要紧: **先建后删**。反过来的话 create_task 失败就两种机制
        // 都没了 —— 用户原本能用的自启被这次操作弄丢, 而提示只是一个气泡。
        // 先建后删最坏是两条并存, 单实例锁兜得住。
        create_task(config_path)?;
        remove_run_entry()?;
        Ok(Enabled::Done)
    } else {
        write_run_entry(config_path)?;
        delete_task_quiet();
        Ok(Enabled::Done)
    }
}

/// 关闭自启, 两种机制都清掉。
pub fn disable() -> Result<(), String> {
    let r = remove_run_entry();
    delete_task_quiet();
    r
}

/// 让实际使用的机制与当前偏好一致。启动时调用一次, 幂等。
///
/// 用途: 用户刚打开"以管理员运行"时还没有权限建任务, 提权重启之后
/// 由这里补上, 并把旧的 Run 项清掉。
pub fn sync(config_path: &Path) -> Result<(), String> {
    if !is_enabled() {
        return Ok(());
    }
    let admin_mode = elevate::wants_admin();
    let via_task = task_exists();

    // 已经在正确的机制上。但任务可能是老版本用 schtasks 命令行建的,
    // 那种任务带着电池限制和 3 天的运行时长上限 —— 机制"一致"归一致,
    // 自启还是时灵时不灵, 所以这里要再看一眼格式版本。
    if admin_mode == via_task {
        if via_task && task_needs_rebuild() {
            if !elevate::is_elevated() {
                debug!("旧格式的自启任务需要管理员权限才能重建, 本次跳过");
                return Ok(());
            }
            info!("自启任务是旧格式建的, 重建一遍以关掉电池限制与运行时长上限");
            return create_task(config_path);
        }
        return Ok(());
    }
    debug!(
        "自启机制需要切换: 当前={}, 期望={}",
        if via_task { "计划任务" } else { "Run 项" },
        if admin_mode {
            "计划任务"
        } else {
            "Run 项"
        }
    );
    enable(config_path).map(|_| ())
}

// ---------- HKCU\Run ----------

/// Run 项存在且指向**当前这个 exe**。
///
/// 不只看键是否存在: 用户可能有多份副本, 注册表里那条若指向另一份,
/// 对本实例来说就不算"已启用"。
fn run_entry_is_self() -> bool {
    let Some(existing) = regutil::read_string(HKEY_CURRENT_USER, RUN_KEY, RUN_VALUE) else {
        return false;
    };
    // 必须和 launch_command 用同一个路径口径, 否则自己写下的项自己认不出来
    let prefix = format!("\"{}\"", paths::stable_exe().display()).to_lowercase();
    existing.to_lowercase().starts_with(&prefix)
}

fn write_run_entry(config_path: &Path) -> Result<(), String> {
    let cmd = launch_command(config_path)?;
    regutil::write_string(HKEY_CURRENT_USER, RUN_KEY, RUN_VALUE, &cmd)
}

fn remove_run_entry() -> Result<(), String> {
    regutil::delete_value(HKEY_CURRENT_USER, RUN_KEY, RUN_VALUE)
}

// ---------- 任务计划程序 ----------

/// 任务是否存在。
///
/// 只有 schtasks 明确回答"没有这个任务"才算不存在。调用不起来、权限不足
/// 这类失败会被记下来但同样返回 false —— 那时说不准, 而误判成"已启用"
/// 会让托盘的勾变成假的; 误判成"未启用"最多是用户再点一次, `/F` 覆盖创建。
fn task_exists() -> bool {
    match schtasks(&["/Query", "/TN", &task_name()]) {
        Ok(_) => true,
        Err(e) if e.code == Some(SCHTASKS_NOT_FOUND) => false,
        Err(e) => {
            warn!("查询自启任务失败, 暂按未启用处理: {e}");
            false
        }
    }
}

fn create_task(config_path: &Path) -> Result<(), String> {
    let xml = task_xml(config_path)?;
    let xml_path = write_temp_xml(&xml)?;

    let result = schtasks(&[
        "/Create",
        "/F", // 已存在则覆盖
        "/TN",
        &task_name(),
        "/XML",
        &xml_path.to_string_lossy(),
    ]);

    // 临时文件删不掉不影响结果, 它在 %TEMP% 里
    let _ = std::fs::remove_file(&xml_path);

    result.map_err(|e| format!("创建自启任务失败: {e}"))?;

    // 记下格式版本。写失败不算错: 后果只是下次启动多重建一次任务。
    if let Err(e) = regutil::write_dword(
        HKEY_CURRENT_USER,
        elevate::APP_KEY,
        VALUE_TASK_VERSION,
        TASK_FORMAT_VERSION,
    ) {
        warn!("记录任务格式版本失败, 下次启动会多重建一次自启任务: {e}");
    }
    Ok(())
}

/// 已建好的任务是不是旧格式的。没有版本记录就当成 1 (schtasks 命令行方式)。
fn task_needs_rebuild() -> bool {
    regutil::read_dword(HKEY_CURRENT_USER, elevate::APP_KEY, VALUE_TASK_VERSION).unwrap_or(0)
        < TASK_FORMAT_VERSION
}

/// 删除任务, 失败只记日志。任务本来不存在也会"失败", 那不算问题。
fn delete_task_quiet() {
    if let Err(e) = schtasks(&["/Delete", "/F", "/TN", &task_name()]) {
        debug!("删除自启任务未成功 (可能本来就没有): {e}");
    }
}

/// schtasks 的失败。带上退出码, 让调用方能区分"任务不存在"和"没跑起来"。
struct SchtasksError {
    /// `None` 表示进程根本没启动 (比如 PATH 里找不到 schtasks)。
    code: Option<i32>,
    message: String,
}

impl std::fmt::Display for SchtasksError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.code {
            Some(c) => write!(f, "{} (退出码 {c})", self.message),
            None => write!(f, "{}", self.message),
        }
    }
}

fn schtasks(args: &[&str]) -> Result<String, SchtasksError> {
    let out = Command::new("schtasks")
        .args(args)
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| SchtasksError {
            code: None,
            message: format!("无法调用 schtasks: {e}"),
        })?;

    if out.status.success() {
        return Ok(String::from_utf8_lossy(&out.stdout).into_owned());
    }
    // schtasks 的中文输出是 OEM 编码, from_utf8_lossy 可能出乱码,
    // 但错误信息只进日志, 够用
    let msg = String::from_utf8_lossy(&out.stderr).trim().to_string();
    Err(SchtasksError {
        code: out.status.code(),
        message: if msg.is_empty() {
            "schtasks 执行失败".to_string()
        } else {
            msg
        },
    })
}

// ---------- 任务 XML ----------

/// 任务定义。见模块头部关于"为什么不用命令行参数"的说明。
///
/// 元素顺序照着任务计划程序自己导出的样子写。schema 用的是 `xs:all`,
/// 顺序本不影响解析, 但和系统导出的保持一致, 日后拿两份 XML 对比最省事。
fn task_xml(config_path: &Path) -> Result<String, String> {
    let exe = paths::stable_exe();
    if exe.as_os_str().is_empty() {
        return Err("无法确定程序路径".into());
    }

    let args = launch_arguments(config_path);
    let arguments_line = if args.is_empty() {
        String::new()
    } else {
        format!("\n      <Arguments>{}</Arguments>", xml_escape(&args))
    };

    // 触发器不带 UserId 的话语义是"任何用户登录时", 而 Principal 又限定成
    // 本用户 —— 单用户机器上侥幸能跑, 多用户机器上就是错的。
    let user_line = match current_user_id() {
        Some(u) => format!("\n      <UserId>{}</UserId>", xml_escape(&u)),
        None => {
            warn!("无法确定当前用户名, 登录触发器将对任何用户生效");
            String::new()
        }
    };

    // 登录触发任务是系统里最早起来的一批进程, 常抢在 explorer 建好任务栏之前;
    // 那时往托盘加图标会失败, 而 main 把托盘初始化失败当致命错误直接退出 ——
    // 自启就成了"设了但重启后不起"。推迟触发, 让 explorer 先就位。
    Ok(format!(
        r#"<?xml version="1.0" encoding="UTF-16"?>
<Task version="1.2" xmlns="http://schemas.microsoft.com/windows/2004/02/mit/task">
  <RegistrationInfo>
    <Description>keyremap 开机自启 (以最高可用权限运行)</Description>
  </RegistrationInfo>
  <Triggers>
    <LogonTrigger>
      <Enabled>true</Enabled>{user_line}
      <Delay>PT10S</Delay>
    </LogonTrigger>
  </Triggers>
  <Principals>
    <Principal id="Author">
      <LogonType>InteractiveToken</LogonType>
      <RunLevel>HighestAvailable</RunLevel>
    </Principal>
  </Principals>
  <Settings>
    <MultipleInstancesPolicy>IgnoreNew</MultipleInstancesPolicy>
    <DisallowStartIfOnBatteries>false</DisallowStartIfOnBatteries>
    <StopIfGoingOnBatteries>false</StopIfGoingOnBatteries>
    <AllowHardTerminate>true</AllowHardTerminate>
    <StartWhenAvailable>false</StartWhenAvailable>
    <RunOnlyIfNetworkAvailable>false</RunOnlyIfNetworkAvailable>
    <IdleSettings>
      <StopOnIdleEnd>false</StopOnIdleEnd>
      <RestartOnIdle>false</RestartOnIdle>
    </IdleSettings>
    <AllowStartOnDemand>true</AllowStartOnDemand>
    <Enabled>true</Enabled>
    <Hidden>false</Hidden>
    <RunOnlyIfIdle>false</RunOnlyIfIdle>
    <WakeToRun>false</WakeToRun>
    <ExecutionTimeLimit>PT0S</ExecutionTimeLimit>
    <Priority>7</Priority>
  </Settings>
  <Actions Context="Author">
    <Exec>
      <Command>{command}</Command>{arguments_line}
    </Exec>
  </Actions>
</Task>
"#,
        command = xml_escape(&exe.display().to_string()),
    ))
}

/// XML 落到临时文件。**必须是 UTF-16 LE 带 BOM** —— schtasks 只认这个,
/// 喂 UTF-8 会得到一句没头没脑的解析错误。
fn write_temp_xml(xml: &str) -> Result<PathBuf, String> {
    let path = std::env::temp_dir().join(format!("{}.xml", task_name()));

    let mut bytes = Vec::with_capacity(xml.len() * 2 + 2);
    bytes.extend_from_slice(&[0xFF, 0xFE]); // UTF-16 LE BOM
    for unit in xml.encode_utf16() {
        bytes.extend_from_slice(&unit.to_le_bytes());
    }

    std::fs::write(&path, &bytes)
        .map_err(|e| format!("写入任务定义 {} 失败: {e}", path.display()))?;
    Ok(path)
}

/// 触发器里的用户标识。任务计划程序接受 `域\用户` 形式并会自己转成 SID。
fn current_user_id() -> Option<String> {
    let user = std::env::var("USERNAME").ok().filter(|s| !s.is_empty())?;
    match std::env::var("USERDOMAIN") {
        Ok(domain) if !domain.is_empty() => Some(format!("{domain}\\{user}")),
        _ => Some(user),
    }
}

/// 路径里出现 `&` 的目录并不罕见, 不转义就是一份坏 XML。
fn xml_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(c),
        }
    }
    out
}

// ---------- 公共 ----------

/// 自启时使用的完整命令行。`HKCU\Run` 存的就是这一个字符串。
///
/// exe 路径走 [`paths::stable_exe`]: 这条路径要留到下次开机, 而 scoop 的
/// 版本目录到那时可能已经不存在了。
fn launch_command(config_path: &Path) -> Result<String, String> {
    let exe = paths::stable_exe();
    if exe.as_os_str().is_empty() {
        return Err("无法确定程序路径".into());
    }
    let args = launch_arguments(config_path);
    Ok(if args.is_empty() {
        format!("\"{}\"", exe.display())
    } else {
        format!("\"{}\" {args}", exe.display())
    })
}

/// 命令行里 exe 之后的部分。计划任务的 `<Arguments>` 要的正是这一段。
///
/// 配置正好是默认查找结果时返回空串: 命令行更短也更不容易出错, 而省掉 `-c`
/// 之后启动的实例会自己重跑一遍查找, 结果和现在一致。
fn launch_arguments(config_path: &Path) -> String {
    if paths::default_config().as_path() == config_path {
        String::new()
    } else {
        format!("-c \"{}\"", config_path.display())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn 默认配置路径不带参数() {
        let cmd = launch_command(&paths::default_config()).unwrap();
        assert!(!cmd.contains("-c"), "默认路径不该带 -c: {cmd}");
        assert!(cmd.starts_with('"') && cmd.ends_with('"'));
        assert!(launch_arguments(&paths::default_config()).is_empty());
    }

    #[test]
    fn 非默认配置路径带上参数() {
        let other = PathBuf::from(r"D:\somewhere\my.toml");
        let cmd = launch_command(&other).unwrap();
        assert!(cmd.contains("-c"), "非默认路径应带 -c: {cmd}");
        assert!(cmd.contains("my.toml"));
        // 路径两侧都要有引号, 否则含空格的路径会被拆开
        assert!(cmd.contains(r#""D:\somewhere\my.toml""#));
    }

    #[test]
    fn 任务名与单实例锁同源() {
        // 两者都按 exe 路径区分, 保证多副本行为一致
        assert_eq!(task_name(), singleton::name_for_current_exe());
        assert!(task_name().starts_with("keyremap-"));
    }

    // ---- 任务 XML ----

    /// 这三项就是 schtasks 命令行方式的全部病灶, 值不对等于自启白设。
    #[test]
    fn xml_关掉了电池限制与运行时长上限() {
        let xml = task_xml(&paths::default_config()).unwrap();
        assert!(xml.contains("<DisallowStartIfOnBatteries>false</DisallowStartIfOnBatteries>"));
        assert!(xml.contains("<StopIfGoingOnBatteries>false</StopIfGoingOnBatteries>"));
        // PT0S = 不限时。省略这个元素会取默认的 3 天, 常驻进程会被杀掉
        assert!(xml.contains("<ExecutionTimeLimit>PT0S</ExecutionTimeLimit>"));
    }

    #[test]
    fn xml_以最高权限在登录时运行() {
        let xml = task_xml(&paths::default_config()).unwrap();
        assert!(xml.contains("<RunLevel>HighestAvailable</RunLevel>"));
        assert!(xml.contains("<LogonTrigger>"));
        assert!(xml.contains("<LogonType>InteractiveToken</LogonType>"));
    }

    #[test]
    fn xml_登录触发器带延迟() {
        // 登录瞬间 explorer 的任务栏往往还没就位, 立即触发会让托盘初始化失败,
        // 程序退出 (见 task_xml 里 Delay 的说明)。这条保证延迟不会被顺手删掉。
        let xml = task_xml(&paths::default_config()).unwrap();
        assert!(xml.contains("<Delay>PT10S</Delay>"));
    }

    #[test]
    fn xml_默认配置时不写arguments() {
        let xml = task_xml(&paths::default_config()).unwrap();
        assert!(!xml.contains("<Arguments>"), "没有参数就不该有这个元素");
        // Command 是纯路径, 不带引号 —— 引号是命令行的事, XML 里分开放
        assert!(!xml.contains("<Command>\""), "Command 不该带引号: {xml}");
    }

    #[test]
    fn xml_自定义配置时命令与参数分开放() {
        let other = PathBuf::from(r"D:\somewhere\my.toml");
        let xml = task_xml(&other).unwrap();
        assert!(xml.contains(r#"<Arguments>-c &quot;D:\somewhere\my.toml&quot;</Arguments>"#));
        // 参数绝不能混进 Command, 那正是命令行方式做错的地方
        assert!(!xml.contains("<Command>-"));
        let command_line = xml
            .lines()
            .find(|l| l.contains("<Command>"))
            .expect("必须有 Command");
        assert!(!command_line.contains("-c"), "Command 里不该有参数");
    }

    #[test]
    fn xml_转义特殊字符() {
        assert_eq!(
            xml_escape(r#"a&b<c>d"e'f"#),
            "a&amp;b&lt;c&gt;d&quot;e&apos;f"
        );
        // 含 & 的目录不罕见, 不转义就是一份坏 XML
        let xml = task_xml(&PathBuf::from(r"D:\a&b\my.toml")).unwrap();
        assert!(xml.contains("a&amp;b"));
        assert!(!xml.contains("a&b"), "裸 & 会让 XML 解析失败");
    }

    #[test]
    fn 临时xml是utf16带bom() {
        let path = write_temp_xml("<Task/>").unwrap();
        let bytes = std::fs::read(&path).unwrap();
        assert_eq!(&bytes[..2], &[0xFF, 0xFE], "schtasks 只认 UTF-16 LE BOM");
        // "<Task/>" 的第一个字符, 小端序
        assert_eq!(&bytes[2..4], &[b'<', 0]);
        let _ = std::fs::remove_file(&path);
    }
}
