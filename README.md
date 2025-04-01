# Chess Engine for [Gambit+](https://github.com/oskaars/Motorola123)

## Building and Running the Engine

### Build
To compile the engine in release mode, use the following command:
```sh
cargo build --release
```

### Run
To execute the engine in release mode, use the following command:
```sh
cargo run --release
```

## Opening Book Integration
The engine supports PGN files for use as opening books. The `BOOK_PATH` variable in the `.env` file can be configured in two ways:

1. **Directory path** - By default, the engine expects `BOOK_PATH` to point to a directory containing opening book files.
2. **Direct PGN file** - Alternatively, you can set `BOOK_PATH` to point directly to a specific PGN file (e.g., `book.pgn`).

If you don't customize the path, the engine will look for a directory named `book.pgn` in the project root.

An example opening book can be accessed [here](https://drive.google.com/file/d/1R5Fyqb-ZCWZhPpyxyvjzTI-D-3QLqLAu/view?usp=drive_link).