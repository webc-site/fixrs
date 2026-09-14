use proc_macro2::Span;

/// 记录一处需要被简化的路径信息
#[derive(Debug, Clone)]
pub struct QualifiedPath {
  /// 原始路径完整字符串（如 "wbase::time::now_secs" 或 "std::io::Error"）
  pub original: String,
  /// 替换后的短路径文本（如 "now_secs" 或 "Error"）
  pub replacement: String,
  /// 应当在文件头部引入的 use 路径（如 "wbase::time::now_secs" 或 "std::io::Error"）
  pub import: String,
  /// 原始路径各段名称列表
  pub segments: Vec<String>,
  /// AST span
  pub span: Span,
}

impl QualifiedPath {
  /// 获取路径末尾的符号标识符（零额外堆分配）
  #[inline]
  pub fn last_ident(&self) -> &str {
    self.segments.last().map_or("", String::as_str)
  }

  /// 获取实际导入符号的末尾名称（如 std::io::Error -> Error）
  #[inline]
  pub fn import_tail(&self) -> &str {
    self.import.rsplit("::").next().unwrap_or_default()
  }

  /// 获取路径中代表核心类型的标识符（用于同名类型冲突检测）
  /// - 若倒数第二项首字母大写（如 Error::new, ErrorKind::Other），核心类型为倒数第二项
  /// - 否则为末尾项（如 std::io::Error -> Error）
  #[inline]
  pub fn core_ident(&self) -> &str {
    let total = self.segments.len();
    if total >= 2
      && let Some(prev) = self.segments.get(total - 2)
      && prev.as_bytes().first().is_some_and(u8::is_ascii_uppercase)
    {
      prev.as_str()
    } else {
      self.last_ident()
    }
  }
}
