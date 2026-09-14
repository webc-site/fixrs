use std::{env, sync::OnceLock};

mod langs;
mod msg;

pub use msg::I18n;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
  Zh,
  En,
  Ja,
  Ko,
  Fr,
  De,
  Es,
  Ru,
  Pt,
  It,
  Ar,
  Hi,
  Vi,
  Th,
  Tr,
  Pl,
  Nl,
  Id,
  Uk,
  Cs,
  Sv,
  El,
  He,
  Ro,
  Hu,
  Da,
  Fi,
  No,
}

static CURRENT_LANG: OnceLock<Lang> = OnceLock::new();
const LOCALE_ENV_VARS: &[&str] = &["LC_ALL", "LC_MESSAGES", "LANGUAGE", "LANG"];

/// 针对单个 Locale 候选项字符串进行启发式解析
/// 规则：
/// 1. 包含 "zh"（不区分大小写）一律命中 Lang::Zh（覆盖 zh_CN, zh_TW, zh_HK, zh-Hans, zh-Hant 等繁简体）
/// 2. 忽略 "c", "posix", "c.utf-8" 等无本地化环境标记
/// 3. 解析主语言代码（提取首个分段），支持 ISO 639-1 (2位)、ISO 639-2/3 (3位) 及常见全称
/// 4. 无法匹配时返回 None
pub fn match_locale_str(locale: &str) -> Option<Lang> {
  let trimmed = locale.trim();
  if trimmed.is_empty() {
    return None;
  }

  // 1. 用户核心规则：凡是包含 "zh"（如 zh_CN, zh_TW, zh_HK, zh-Hans, zh-Hant, zh-SG）一律视为中文
  if trimmed
    .as_bytes()
    .windows(2)
    .any(|w| w.eq_ignore_ascii_case(b"zh"))
  {
    return Some(Lang::Zh);
  }

  // 2. 忽略 POSIX 默认的 C/POSIX 本地化
  let first_part = trimmed
    .split(['.', '@', '_', '-', ':'])
    .next()
    .unwrap_or("")
    .trim();

  if first_part.eq_ignore_ascii_case("c") || first_part.eq_ignore_ascii_case("posix") {
    return None;
  }

  // 3. 启发式前缀与别名高效匹配（单次栈内存切片模式匹配，零堆分配）
  let mut buf = [0u8; 16];
  if first_part.is_empty() || first_part.len() > buf.len() {
    return None;
  }
  let s = &mut buf[..first_part.len()];
  s.copy_from_slice(first_part.as_bytes());
  s.make_ascii_lowercase();

  match &*s {
    b"en" | b"eng" | b"english" => Some(Lang::En),
    b"ja" | b"jpn" | b"japanese" => Some(Lang::Ja),
    b"ko" | b"kor" | b"korean" => Some(Lang::Ko),
    b"fr" | b"fra" | b"fre" | b"french" => Some(Lang::Fr),
    b"de" | b"deu" | b"ger" | b"german" => Some(Lang::De),
    b"es" | b"spa" | b"spanish" => Some(Lang::Es),
    b"ru" | b"rus" | b"russian" => Some(Lang::Ru),
    b"pt" | b"por" | b"portuguese" => Some(Lang::Pt),
    b"it" | b"ita" | b"italian" => Some(Lang::It),
    b"ar" | b"ara" | b"arabic" => Some(Lang::Ar),
    b"hi" | b"hin" | b"hindi" => Some(Lang::Hi),
    b"vi" | b"vie" | b"vietnamese" => Some(Lang::Vi),
    b"th" | b"tha" | b"thai" => Some(Lang::Th),
    b"tr" | b"tur" | b"turkish" => Some(Lang::Tr),
    b"pl" | b"pol" | b"polish" => Some(Lang::Pl),
    b"nl" | b"nld" | b"dut" | b"dutch" => Some(Lang::Nl),
    b"id" | b"ind" | b"in" | b"indonesian" => Some(Lang::Id),
    b"uk" | b"ukr" | b"ukrainian" => Some(Lang::Uk),
    b"cs" | b"ces" | b"cze" | b"czech" => Some(Lang::Cs),
    b"sv" | b"swe" | b"swedish" => Some(Lang::Sv),
    b"el" | b"ell" | b"gre" | b"greek" => Some(Lang::El),
    b"he" | b"heb" | b"iw" | b"hebrew" => Some(Lang::He),
    b"ro" | b"ron" | b"rum" | b"romanian" => Some(Lang::Ro),
    b"hu" | b"hun" | b"hungarian" => Some(Lang::Hu),
    b"da" | b"dan" | b"danish" => Some(Lang::Da),
    b"fi" | b"fin" | b"finnish" => Some(Lang::Fi),
    b"no" | b"nor" | b"nb" | b"nn" | b"norwegian" => Some(Lang::No),
    _ => None,
  }
}

/// 检测当前操作系统或终端环境的语言偏好
/// 依次检查 LC_ALL、LC_MESSAGES、LANGUAGE、LANG 等环境变量
/// 若匹配不上任何已支持语言，默认安全回退到英文 (Lang::En)
pub fn detect_system_lang() -> Lang {
  for var in LOCALE_ENV_VARS {
    if let Ok(val) = env::var(var) {
      for candidate in val.split(':') {
        if let Some(lang) = match_locale_str(candidate) {
          return lang;
        }
      }
    }
  }
  // 匹配不上，默认使用英文
  Lang::En
}

/// 获取当前激活的语言
pub fn current_lang() -> Lang {
  *CURRENT_LANG.get_or_init(detect_system_lang)
}

/// 允许在测试或特殊场景下主动设置当前语言
pub fn set_lang(lang: Lang) {
  let _ = CURRENT_LANG.set(lang);
}

/// 获取当前语言包的静态常量引用（零虚表、零堆分配、常数时间访问）
#[inline]
pub fn msg() -> &'static I18n {
  msg_for_lang(current_lang())
}

/// 获取指定语言的常量包引用
#[inline]
pub fn msg_for_lang(lang: Lang) -> &'static I18n {
  match lang {
    Lang::Zh => &langs::ZH,
    Lang::En => &langs::EN,
    Lang::Ja => &langs::JA,
    Lang::Ko => &langs::KO,
    Lang::Fr => &langs::FR,
    Lang::De => &langs::DE,
    Lang::Es => &langs::ES,
    Lang::Ru => &langs::RU,
    Lang::Pt => &langs::PT,
    Lang::It => &langs::IT,
    Lang::Ar => &langs::AR,
    Lang::Hi => &langs::HI,
    Lang::Vi => &langs::VI,
    Lang::Th => &langs::TH,
    Lang::Tr => &langs::TR,
    Lang::Pl => &langs::PL,
    Lang::Nl => &langs::NL,
    Lang::Id => &langs::ID,
    Lang::Uk => &langs::UK,
    Lang::Cs => &langs::CS,
    Lang::Sv => &langs::SV,
    Lang::El => &langs::EL,
    Lang::He => &langs::HE,
    Lang::Ro => &langs::RO,
    Lang::Hu => &langs::HU,
    Lang::Da => &langs::DA,
    Lang::Fi => &langs::FI,
    Lang::No => &langs::NO,
  }
}
