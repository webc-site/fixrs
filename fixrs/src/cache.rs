use std::{
  fs::{self, File},
  io::{self, Write},
  path::{Path, PathBuf},
  process,
  time::UNIX_EPOCH,
};

use bitcode::{Decode, Encode};

use crate::hash::{HashMap, hash64};

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub struct FileMeta {
  pub size: u64,
  pub mtime_secs: u64,
  pub mtime_nanos: u32,
}

impl FileMeta {
  pub fn from_path(path: &Path) -> Option<Self> {
    let metadata = fs::metadata(path).ok()?;
    let size = metadata.len();
    let mtime = metadata.modified().ok()?;
    let duration = mtime.duration_since(UNIX_EPOCH).ok()?;
    Some(Self {
      size,
      mtime_secs: duration.as_secs(),
      mtime_nanos: duration.subsec_nanos(),
    })
  }
}

#[derive(Encode, Decode, Default, Debug)]
pub struct ProjectCache {
  pub config_hash: u64,
  pub clean_files: HashMap<String, FileMeta>,
}

/// 向上查找包含任一指定文件名的完整路径（就近优先，单次遍历，零多余堆分配）
pub fn find_upward_files(start: &Path, file_names: &[&str]) -> Option<PathBuf> {
  let start_dir = if start.is_dir() {
    start
  } else {
    start.parent().unwrap_or(Path::new("."))
  };

  let mut current = start_dir
    .canonicalize()
    .unwrap_or_else(|_| start_dir.to_path_buf());
  loop {
    for &name in file_names {
      current.push(name);
      if current.is_file() {
        return Some(current);
      }
      current.pop();
    }
    if !current.pop() {
      break;
    }
  }

  None
}

/// 向上查找包含指定文件名的完整路径
pub fn find_upward_file(start: &Path, file_name: &str) -> Option<PathBuf> {
  find_upward_files(start, &[file_name])
}

/// 从指定路径向上查找最近的包含 Cargo.toml 的工程根目录（原地推导，零多余 PathBuf 分配）
pub fn find_project_root(start: &Path) -> PathBuf {
  find_upward_file(start, "Cargo.toml")
    .map(|mut p| {
      p.pop();
      p
    })
    .unwrap_or_else(|| start.to_path_buf())
}

/// 计算工程的缓存文件存储路径（基于 dirs::cache_dir 与项目根路径哈希，零多余堆分配）
pub fn get_cache_file_path(project_root: &Path) -> Option<PathBuf> {
  let cache_dir = dirs::cache_dir()?.join("fixrs");
  let canonical = project_root
    .canonicalize()
    .unwrap_or_else(|_| project_root.to_path_buf());
  let hash = hash64(canonical.as_os_str().as_encoded_bytes());

  Some(cache_dir.join(format!("{hash:016x}.bin")))
}

/// 从缓存文件加载 ProjectCache
pub fn load_cache(cache_path: &Path, expected_config_hash: u64) -> ProjectCache {
  let Ok(bytes) = fs::read(cache_path) else {
    return ProjectCache {
      config_hash: expected_config_hash,
      clean_files: HashMap::default(),
    };
  };

  match bitcode::decode::<ProjectCache>(&bytes) {
    Ok(cache) if cache.config_hash == expected_config_hash => cache,
    _ => ProjectCache {
      config_hash: expected_config_hash,
      clean_files: HashMap::default(),
    },
  }
}

/// 将 ProjectCache 原子持久化到缓存文件
pub fn save_cache(cache_path: &Path, cache: &ProjectCache) -> io::Result<()> {
  let Some(parent) = cache_path.parent() else {
    return Ok(());
  };
  fs::create_dir_all(parent)?;

  let encoded = bitcode::encode(cache);
  let temp_path = parent.join(format!(".tmp_{}", process::id()));

  {
    let mut file = File::create(&temp_path)?;
    file.write_all(&encoded)?;
    file.flush()?;
  }

  fs::rename(&temp_path, cache_path)?;
  Ok(())
}
