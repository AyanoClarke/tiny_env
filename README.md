# TinyENV

This is a tiny environment management script.

## Build

`tiny_env` is written in Rust, so you'll need to grab a [Rust installation](https://www.rust-lang.org/tools/install) to compile it.

```sh
cargo build --release
```

## Usage

An example config is in the `examples` folder.

```toml
conda = "/work/software/conda3/"

[minimap2]
PATH = "/work/software/minimap2/bin"

[hts]
conda = "/work/software/hts/hts-conda3"

[samtools]
require = ["hts"]
```

When running command `tiny_env -c example.config.toml samtools minimap2`, will genreate a file named `env.profile`.

```sh
# software config is "examples/example.config.toml" activate modules {"hts", "samtools", "minimap2"}
source /work/software/conda3/bin/activate /work/software/hts/hts-conda3
export PATH=/work/software/minimap2/bin:$PATH
```

## TODO
- [ ] Find base conda environment by conda cli.
- [ ] Add `conflict` keywords.
- [ ] Add `version` to deal with different versions.
