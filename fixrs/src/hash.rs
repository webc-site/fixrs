use std::{
  collections::{HashMap as StdHashMap, HashSet as StdHashSet},
  hash::{BuildHasher, Hasher},
};

pub struct MuseHasher(museair::Hasher);

impl Hasher for MuseHasher {
  #[inline]
  fn finish(&self) -> u64 {
    self.0.finish()
  }

  #[inline]
  fn write(&mut self, bytes: &[u8]) {
    self.0.write(bytes);
  }
}

#[derive(Clone, Copy, Default, Debug)]
pub struct MuseBuildHasher;

impl BuildHasher for MuseBuildHasher {
  type Hasher = MuseHasher;

  #[inline]
  fn build_hasher(&self) -> Self::Hasher {
    new_hasher()
  }
}

pub type HashMap<K, V> = StdHashMap<K, V, MuseBuildHasher>;
pub type HashSet<T> = StdHashSet<T, MuseBuildHasher>;

/// 创建基于 0 种子的 MuseHasher 实例
#[inline]
pub fn new_hasher() -> MuseHasher {
  MuseHasher(museair::Hasher::with_seed(0))
}

/// 单次快速哈希计算
#[inline]
pub fn hash64(bytes: &[u8]) -> u64 {
  museair::hash(bytes, 0)
}
