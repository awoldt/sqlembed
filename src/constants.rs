// file extensions that can all use the File read_to_string() way of parsing
pub const TEXT_EXTENSIONS: &[&str] = &[
    "txt", "md", "json", "jsonl", "yaml", "yml", "toml", "ini", "cfg", "conf", "csv", "tsv", "xml",
    "html", "htm", "sql", "log", "rs", "go", "py", "java", "kt", "scala", "cs", "c", "h", "cpp",
    "hpp", "swift", "dart", "zig", "lua", "rb", "php", "pl", "r", "jl", "hs", "js", "jsx", "mjs",
    "cjs", "ts", "tsx", "css", "scss", "sass", "less", "sh", "bash", "zsh", "fish", "ps1", "bat",
    "cmd", "mk", "cmake", "tf", "tfvars", "hcl", "proto", "graphql", "gql", "tex", "bib",
    "feature", "http", "asm", "s", "patch", "diff",
];

// file extensions that have custom parsing logic
pub const DOCUMENT_EXTENSIONS: &[&str] = &["pdf", "docx", "pptx"];
