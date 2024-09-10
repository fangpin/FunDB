pub struct Options {
    pub cmp: Rc<Box<dyn Cmp>>,
    pub env: Rc<Box<dyn Env>>,
    pub log: Option<Shared<Logger>>,
    pub create_if_missing: bool,
    pub error_if_exists: bool,
    pub paranoid_checks: bool,
    pub write_buffer_size: usize,
    pub max_open_files: usize,
    pub max_file_size: usize,
    pub block_cache: Shared<Cache<Block>>,
    pub block_size: usize,
    pub block_restart_interval: usize,
    /// Compressor id in compressor list
    ///
    /// Note: you have to open a database with the same compression type as it was written to, in
    /// order to not lose data! (this is a bug and will be fixed)
    pub compressor: u8,

    pub compressor_list: Rc<CompressorList>,
    pub reuse_logs: bool,
    pub reuse_manifest: bool,
    pub filter_policy: filter::BoxedFilterPolicy,
}