## Film Borders in WASM

If you have a modern browser, you can use the live WASM web version [here](https://film-borders.romnn.com).

#### Installation
You can use the [web application](https://film-borders.romnn.com) that uses WASM or the CLI tool for batch processing.

To install the CLI, run
```bash
cargo install --git https://github.com/romnn/wasm-film-borders.git --bin film-borders
```

For local testing, you can also install the current version locally:
```bash
cargo install --bin film-borders --path .
```

#### Benchmarking
```bash
sudo cargo flamegraph -o my_flamegraph.svg -- apply --image ./samples/sample1.jpg --output ./output/sample1.png --border 0 --scale 1.00
```

#### Usage
```bash
film-borders apply --image ~/Downloads/testscan.jpg --width 2000 --height 1500 --border 10 --rotate 90
```

For a list of options, see
```bash
film-borders apply --help
```
