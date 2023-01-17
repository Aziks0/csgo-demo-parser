# Generate rust code from `.proto`

This script uses
[`rust-script`](https://github.com/fornwall/rust-script)
and
[`protobuf-codegen`](https://github.com/stepancheg/rust-protobuf/tree/master/protobuf-codegen)
to generate the rust code.

The proto files come from
[`SteamDB`](https://github.com/SteamDatabase/GameTracking-CSGO/tree/master/Protobufs)
.

## Steps

Install [`rust-script`](https://github.com/fornwall/rust-script)

```sh
cargo install rust-script
```

Execute `./generate.ers` with `rust-script`

```sh
./generate.ers
```

or _(should be used by `Windows` users)_

```sh
rust-script ./generate.ers
```
