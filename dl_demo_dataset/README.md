# Download and decompress `.dem` test files

This script uses
[`rust-script`](https://github.com/fornwall/rust-script),
[`git-lfs`](https://git-lfs.com)
and
[`zstd-rs`](https://github.com/gyscos/zstd-rs)
to download and decompress the `.dem` files used in tests.

## Steps

Install [`rust-script`](https://github.com/fornwall/rust-script)

```sh
cargo install rust-script
```

Install [`git-lfs`](https://git-lfs.com)

```sh
git lfs install
```

Execute `./download.ers` with `rust-script`

```sh
./download.ers
```

or _(should be used by `Windows` users)_

```sh
rust-script ./download.ers
```
