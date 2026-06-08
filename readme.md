Parse files and embed text into chunks that can be inserted into a postgresql database to query against with a single command

You must have the pgvector extension installed on the target database for insert queries to work

## Commands

### `chunk`

The final sql query generated will include: 
- files table 
- chunks table

Each chunk record will map to the corresponding file record

```bash
ezvector chunk [path] [--exts <exts>] [--model <model>]
```

| Argument / Flag | Required | Description |
|---|---|---|
| `path` | no | Directory or file to parse. Relative to cwd. Defaults to cwd. |
| `--exts` | no | Comma-separated file extensions to include (e.g. `pdf,txt`). Defaults to all supported extensions. |
| `--model` | no | Embedding model to use. Defaults to `BGESmallENV15`. Run `ezvector list` to see options. |

Supported file extensions: `txt`, `pdf`, `docx`, `pptx`, `md`, `json`

**Examples:**

```bash
ezvector chunk
ezvector chunk ./docs
ezvector chunk ./docs --exts pdf,md
ezvector chunk ./docs --exts pdf --model BGESmallENV15
```

### `list`

List supported embedding models.

```bash
ezvector list
```
