workspace(name = "rules_ebuild")

load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive", "http_file")

http_archive(
    name = "io_bazel_rules_go",
    sha256 = "16e9fca53ed6bd4ff4ad76facc9b7b651a89db1689a2877d6fd7b82aa824e366",
    urls = [
        "https://mirror.bazel.build/github.com/bazelbuild/rules_go/releases/download/v0.34.0/rules_go-v0.34.0.zip",
        "https://github.com/bazelbuild/rules_go/releases/download/v0.34.0/rules_go-v0.34.0.zip",
    ],
)

http_archive(
    name = "bazel_gazelle",
    sha256 = "501deb3d5695ab658e82f6f6f549ba681ea3ca2a5fb7911154b5aa45596183fa",
    urls = [
        "https://mirror.bazel.build/github.com/bazelbuild/bazel-gazelle/releases/download/v0.26.0/bazel-gazelle-v0.26.0.tar.gz",
        "https://github.com/bazelbuild/bazel-gazelle/releases/download/v0.26.0/bazel-gazelle-v0.26.0.tar.gz",
    ],
)

load("@io_bazel_rules_go//go:deps.bzl", "go_register_toolchains", "go_rules_dependencies")
load("@bazel_gazelle//:deps.bzl", "gazelle_dependencies", "go_repository")

go_repository(
    name = "com_github_urfave_cli",
    importpath = "github.com/urfave/cli",
    sum = "h1:cv3/KhXGBGjEXLC4bH0sLuJ9BewaAbpk5oyMOveu4pw=",
    version = "v1.22.9",
)

go_repository(
    name = "com_github_cpuguy83_go_md2man_v2",
    importpath = "github.com/cpuguy83/go-md2man/v2",
    sum = "h1:p1EgwI/C7NhT0JmVkwCD2ZBK8j4aeHQX2pMHHBfMQ6w=",
    version = "v2.0.2",
)

go_repository(
    name = "com_github_burntsushi_toml",
    importpath = "github.com/BurntSushi/toml",
    sum = "h1:Rt8g24XnyGTyglgET/PRUNlrUeu9F5L+7FilkXfZgs0=",
    version = "v1.2.0",
)

go_repository(
    name = "com_github_xrash_smetrics",
    importpath = "github.com/xrash/smetrics",
    sum = "h1:bAn7/zixMGCfxrRTfdpNzjtPYqr8smhKouy9mxVdGPU=",
    version = "v0.0.0-20201216005158-039620a65673",
)

go_repository(
    name = "org_golang_x_text",
    importpath = "golang.org/x/text",
    sum = "h1:olpwvP2KacW1ZWvsR7uQhoyTYvKAupfQrRGBFM352Gk=",
    version = "v0.3.7",
)

go_repository(
    name = "in_gopkg_yaml_v3",
    importpath = "gopkg.in/yaml.v3",
    sum = "h1:fxVm/GzAzEWqLHuvctI91KS9hhNmmWOoWu0XTYJS7CA=",
    version = "v3.0.1",
)

go_repository(
    name = "com_github_russross_blackfriday_v2",
    importpath = "github.com/russross/blackfriday/v2",
    sum = "h1:JIOH55/0cWyOuilr9/qlrm0BSXldqnqwMsf35Ld67mk=",
    version = "v2.1.0",
)

go_repository(
    name = "org_golang_x_sys",
    importpath = "golang.org/x/sys",
    sum = "h1:Sv5ogFZatcgIMMtBSTTAgMYsicp25MXBubjXNDKwm80=",
    version = "v0.0.0-20220731174439-a90be440212d",
)

go_rules_dependencies()

go_register_toolchains(version = "1.18.3")

gazelle_dependencies()

http_file(
    name = "chromiumos_sdk",
    urls = ["https://commondatastorage.googleapis.com/chromiumos-sdk/cros-sdk-2022.07.30.133332.tar.xz"],
)

http_file(
    name = "ethtool_4_13",
    urls = ["https://commondatastorage.googleapis.com/chromeos-mirror/gentoo/distfiles/ethtool-4.13.tar.xz"],
)

http_file(
    name = "elt-patches_20210924",
    urls = ["https://commondatastorage.googleapis.com/chromeos-mirror/gentoo/distfiles/elt-patches-20210924.tar.xz"],
)

http_file(
    name = "dumb_init",
    executable = True,
    urls = ["https://github.com/Yelp/dumb-init/releases/download/v1.2.5/dumb-init_1.2.5_x86_64"],
    sha256 = "e874b55f3279ca41415d290c512a7ba9d08f98041b28ae7c2acb19a545f1c4df",
)
