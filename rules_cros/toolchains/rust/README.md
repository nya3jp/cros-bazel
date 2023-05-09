# Working with rust

Our rust toolchain is very different from cargo, and fits into bazel's norms.

## Creating a new 1st-party crate
Similar to any other language, create a `.rs` file, and then create a `rust_library/binary` target which lists that file as it's `srcs`.

## Adding dependencies on crates
#### 1st-party crates
Add the corresponding target to your deps.

#### 3rd-party crates
Add a dep on `@crates//:<crate name>` (for chromeos) or `@alchemy-crates//:<crate name>` (for infra).

If the crate doesn't exist, or if the version needs to be updated, then:

##### @alchemy-crates
Modify the `//rules_cros/toolchains/rust/alchemy_crates/Cargo.toml`, then cd to the directory that contains it and run `cargo update --workspace` to update the lockfile.


##### @crates
Modify `//third_party/rust_crates/projects/.../Cargo.toml`, then run `third_party/rust_crates/vendor.py`.
