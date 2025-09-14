# Command Line Interface (CLI)

The Allay is running in production mode by default. Set the `ALLAY_ENV` environment variable to `dev` or `development` to enable Development mode.

```bash
export ALLAY_ENV=dev
# or
export ALLAY_ENV=development
```

To create and start the Allay server, use the following command:

```bash
cargo run --bin=allay --package=allay-cli -- new <path> [options]
cd <path>
cargo run --bin=allay --package=allay-cli -- server [options]
```

or using the `root` option:

```bash
cargo run --bin=allay --package=allay-cli -- new <path> [options]
cargo run --bin=allay --package=allay-cli -- server --root=<path> [options]
```
