use crate::i18n::msg::I18n;

pub const ES: I18n = I18n {
  about: "Herramienta CLI para reemplazar rutas absolutas de Rust por declaraciones use (corrige clippy::absolute_paths)",
  path: "Ruta del archivo o directorio a procesar (por defecto busca Cargo.toml hacia arriba)",
  dry_run: "Modo de prueba preliminar sin modificar los archivos",
  write: "Modificar archivos in situ y ejecutar rustfmt (por defecto)",
  check: "Modo de comprobación: salir con código no nulo si se exceden los límites (para CI)",
  max_segments: "Segmentos máximos permitidos antes de simplificar (por defecto: 2)",
  keep_segments: "Número de segmentos finales a conservar (por defecto: 1; 2 para module::item)",
  allow_crate: "Lista blanca de crates autorizados para mantener rutas absolutas",
  extra_crate: "Crates conocidos adicionales especificados explícitamente",
  quiet: "Modo silencioso, oculta los detalles de las modificaciones",
  show: "Modo de visualización, fuerza la impresión de detalles modificados",
  verbose: "Mostrar registros de procesamiento detallados",
  no_cache: "Desactivar la caché incremental y forzar una nueva comprobación completa",

  found_exceeding_limit: |count| {
    format!("Se encontraron {count} archivo(s) con rutas que superan el límite")
  },
  can_be_simplified: |count| {
    format!("\n{count} archivo(s) se pueden simplificar, ejecute sin `--dry-run` para aplicar")
  },
  failed_init_runtime: "Error al inicializar el entorno de ejecución compio",
  error_processing_file: |file, err| format!("[warn] Error al procesar {file}: {err}, omitido"),
};
