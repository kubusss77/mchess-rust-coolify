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
The engine supports PGN files for use as opening books. By default, the engine will look for a file named `book.pgn`. You can change the file path by modifying the `BOOK_PATH` variable in the `.env` file.

An example opening book can be accessed [here](https://drive.google.com/file/d/1R5Fyqb-ZCWZhPpyxyvjzTI-D-3QLqLAu/view?usp=drive_link).
