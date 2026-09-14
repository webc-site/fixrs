use crate::i18n::msg::I18n;

pub const KO: I18n = I18n {
  about: "Rust 소스 코드의 긴 절대 경로를 최상위 use 문으로 자동 치환하여 clippy::absolute_paths를 수정",
  path: "검사 또는 처리할 파일/디렉터리 경로 (기본값: 상위 방향으로 Cargo.toml 자동 탐색)",
  dry_run: "시뮬레이션 모드: 파일 수정 없이 미리보기만 수행",
  write: "파일을 직접 수정하고 rustfmt 자동 실행 (기본값)",
  check: "검사 모드: 제한 초과 경로 발견 시 0이 아닌 상태 코드로 종료 (CI용)",
  max_segments: "허용되는 최대 경로 세그먼트 수 (기본값: 2, clippy::absolute_paths와 일치)",
  keep_segments: "치환 후 유지할 끝 세그먼트 수 (기본값: 1, 2 지정 시 module::item 유지)",
  allow_crate: "절대 경로 유지를 허용할 크레이트 화이트리스트 (예: std, core)",
  extra_crate: "명시적으로 지정할 추가 크레이트 목록",
  quiet: "조용한 모드: 세부 수정 내역 출력 안 함 (비대화형 환경에서 기본값)",
  show: "표시 모드: 세부 수정 내역 강제 출력",
  verbose: "상세 처리 로그 출력",
  no_cache: "증분 캐시를 비활성화하고 전체 다시 검사",

  found_exceeding_limit: |count| {
    format!("제한을 초과하는 절대 경로가 포함된 파일 {count}개를 찾았습니다")
  },
  can_be_simplified: |count| {
    format!(
      "\n총 {count}개 파일을 단순화할 수 있습니다. 변경사항을 적용하려면 `--dry-run` 없이 실행하세요"
    )
  },
  failed_init_runtime: "compio 런타임 초기화 실패",
  error_processing_file: |file, err| format!("[warn] 파일 {file} 처리 중 오류 발생: {err}, 건너뜀"),
};
