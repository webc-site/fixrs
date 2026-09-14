use crate::i18n::msg::I18n;

pub const PT: I18n = I18n {
  about: "Ferramenta CLI para substituir caminhos absolutos do Rust por declarações use (corrige clippy::absolute_paths)",
  path: "Caminho para o arquivo ou diretório a processar (busca Cargo.toml para cima por padrão)",
  dry_run: "Modo de simulação sem modificar os arquivos reais",
  write: "Modificar arquivos no local e executar rustfmt (padrão)",
  check: "Modo de verificação: sair com código diferente de zero se limites forem excedidos (para CI)",
  max_segments: "Segmentos máximos de caminho permitidos antes da simplificação (padrão: 2)",
  keep_segments: "Número de segmentos finais a manter (padrão: 1; 2 para manter module::item)",
  allow_crate: "Lista de permissões de crates autorizadas a manter caminhos absolutos",
  extra_crate: "Crates conhecidas adicionais especificadas explicitamente",
  quiet: "Modo silencioso, oculta detalhes das modificações",
  show: "Modo de exibição, força a impressão dos detalhes modificados",
  verbose: "Exibir logs de processamento detalhados",
  no_cache: "Desativar cache incremental e forçar nova verificação completa",

  found_exceeding_limit: |count| {
    format!("{count} arquivo(s) com caminhos que excedem o limite encontrados")
  },
  can_be_simplified: |count| {
    format!("\n{count} arquivo(s) podem ser simplificados, execute sem `--dry-run` para aplicar")
  },
  failed_init_runtime: "Falha ao inicializar o runtime compio",
  error_processing_file: |file, err| format!("[warn] Erro ao processar {file}: {err}, ignorado"),
};
