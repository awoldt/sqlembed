Parse file text and embed into chunks that can be inserted into a postgresql database all with a single command. It can scan an entire directory or just a single file. All text embedding is run on your machine and does not require any third party api key.

You must have the pgvector extension installed on the target database for insert queries to work.

## Commands

### `chunk`

The final sql query generated will include two tables creations and inserts for each: 
- files 
- chunks

```bash
ezvector chunk [--path <path>] [--exts <exts>] [--model <model>] [--size <size>] [--output <output>]
```

| Flag | Required | Description |
|---|---|---|
| `--path` | no | Directory or file to parse. Relative to cwd. Defaults to cwd. |
| `--exts` | no | Comma-separated file extensions to include (e.g. `pdf,txt`). Defaults to all supported extensions. |
| `--model` | no | Embedding model to use. Defaults to `BGESmallENV15`. Run `ezvector list` to see options. |
| `--size` | no | Number of words per chunk. Defaults to `250`. |
| `--output` | no | Output SQL filename without extension. Defaults to `output` (writes `output.sql`). |

Supported file extensions: `txt`, `pdf`, `docx`, `pptx`, `md`, `json`

**Examples:**

```bash
ezvector chunk
ezvector chunk --path ./docs
ezvector chunk --path ./docs --exts pdf,md
ezvector chunk --path ./docs --exts pdf --model BGESmallENV15
ezvector chunk --path ./docs --size 500 --output my-data
```

### `list`

List supported embedding models.

```bash
ezvector list
```
