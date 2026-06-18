Parse files, embed text, and store vector records in your sql databases all with a single command. All embeddings are done on local machine, no need for third party API calls. 

Supports:
- 🐘 Postgres (requires pgvector extention installed)
- 🐬 MySQL v9.0+

## Commands
### `chunk`

```bash
sqlembed chunk --database-url <url> [--path <path>] [--exts <exts>] [--model <model>] [--size <size>] [--require-ssl]
```

| Flag | Required | Description |
|---|---|---|
| `--database-url` | yes | Database connection string. Must start with `postgres` or `mysql` |
| `--path` | no | Directory or file to parse. Relative to cwd. Defaults to cwd |
| `--exts` | no | Comma-separated file extensions to include (e.g. `pdf,txt`). Defaults to all supported extensions |
| `--model` | no | Embedding model to use. Defaults to `BGESmallENV15`. Run `sqlembed list models` to see options |
| `--size` | no | Number of words per chunk. Defaults to `250` |
| `--require-ssl` | no | Enable SSL/TLS when connecting to the database |

**Examples:**

```bash
sqlembed chunk --database-url postgres://user:pass@localhost/mydb
sqlembed chunk --database-url mysql://user:pass@localhost/mydb --path ./docs
sqlembed chunk --database-url postgres://user:pass@localhost/mydb --path ./docs --exts pdf,md
sqlembed chunk --database-url postgres://user:pass@localhost/mydb --path ./docs --exts pdf --model BGESmallENV15
sqlembed chunk --database-url postgres://user:pass@localhost/mydb --path ./docs --size 500 --require-ssl
```

### `list`
```sqlembed list files``` - List all supported files that can be embedded

```sqlembed list models``` - List all text embedding models available
