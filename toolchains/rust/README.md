# Working with rust

Our rust toolchain is built on top of cargo. Any packages you build with bazel should also be able to be built with cargo, and the source of truth is Cargo.toml files.

## Creating a new 1st-party crate
To create a crate called example_crate in the bazel/toolchains/rust/examples directory, you would run the following:

`bazel run //bazel/toolchains/rust:cargo_init -- --[lib/bin] //bazel/toolchains/rust/examples/example_crate`

If the bazel build target already existed, you can instead run:

`bazel run //bazel/toolchains/rust:cargo_init -- --[lib/bin] //bazel/toolchains/rust/examples/example_crate:example_crate_rust`

If you've already created a crate with the cargo tool, you can add the `--existing` flag to modify a crate that already exists.

## Adding dependencies on crates
Add the following line to your Cargo.toml file, then run `bazel run //bazel/toolchains/rust:update_crates -- --repin`.

`<crate>.workspace = true`


#### 3rd-party crates
If you get an error saying that the crate doesn't exist, it's probably the first time anyone's used this crate.
* Add it to `//:Cargo.toml`
* Run `bazel run //bazel/toolchains/rust:update_crates -- --repin`.

#### 1st-party crates
You will also need to depend on the bazel target in your BUILD rule.
```
rust_binary_crate(
    name = "use_local_crate",
    deps = ["//bazel/toolchains/rust/examples/local_crate"],
)
```
