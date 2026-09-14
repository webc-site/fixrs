use crate::i18n::msg::I18n;

pub const JA: I18n = I18n {
  about: "Rustソースコード内の冗長な絶対修飾パスをトップレベルuseに自動置換し、clippy::absolute_pathsを修正",
  path: "処理対象のファイルまたはディレクトリパス（デフォルトはCargo.tomlを上位方向に自動検索）",
  dry_run: "ドライラン（変更プレビューのみ行い、ファイルは直接変更しない）",
  write: "ファイルをインプレースで更新し、自動でrustfmtを実行（デフォルト）",
  check: "チェックモード：制限超過パスを検出した場合に非ゼロ終了コードで終了（CI向け）",
  max_segments: "許容される最大パス階層数（デフォルト: 2、clippy::absolute_pathsと同一）",
  keep_segments: "置換後に残す末尾の階層数（デフォルト: 1、2を指定するとmodule::itemを維持）",
  allow_crate: "絶対パスを許容するクレートのホワイトリスト（例: std, core）",
  extra_crate: "明示的に指定する追加の既知クレート一覧（依存関係の補完用）",
  quiet: "サイレントモード、詳細な変更出力を抑制（非端末環境ではデフォルト有効）",
  show: "詳細表示モード、変更詳細を強制出力（非対話環境での静音動作を上書き）",
  verbose: "詳細な処理ログを出力",
  no_cache: "増分キャッシュを無効化し、全ファイルを再チェック",

  found_exceeding_limit: |count| {
    format!("制限を超える修飾パスを含むファイルが {count} 件見つかりました")
  },
  can_be_simplified: |count| {
    format!(
      "\n{count} 件のファイルが単純化可能です。変更を適用するには `--dry-run` なしで実行してください"
    )
  },
  failed_init_runtime: "compioランタイムの初期化に失敗しました",
  error_processing_file: |file, err| {
    format!("[warn] ファイル {file} の処理中にエラーが発生しました: {err}（スキップ）")
  },
};
