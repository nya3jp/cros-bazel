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
    sum = "h1:U+s90UTSYgptZMwQh2aRr3LuazLJIa+Pg3Kc1ylSYVY=",
    version = "v2.0.0-20190314233015-f79a8a8ca69d",
)

go_repository(
    name = "com_github_burntsushi_toml",
    importpath = "github.com/BurntSushi/toml",
    sum = "h1:WXkYYl6Yr3qBf1K79EBnL4mak0OimBfB0XUf9Vl28OQ=",
    version = "v0.3.1",
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
    sum = "h1:lPqVAte+HuHNfhJ/0LC98ESWRz8afy9tM/0RK8m9o+Q=",
    version = "v2.0.1",
)

go_repository(
    name = "org_golang_x_sys",
    importpath = "golang.org/x/sys",
    sum = "h1:2QkjZIsXupsJbJIdSjjUOgWK3aEtzyuh2mPt3l/CkeU=",
    version = "v0.0.0-20220811171246-fbc7d0a398ab",
)

go_repository(
    name = "com_github_alessio_shellescape",
    importpath = "github.com/alessio/shellescape",
    sum = "h1:V7yhSDDn8LP4lc4jS8pFkt0zCnzVJlG5JXy9BVKJUX0=",
    version = "v1.4.1",
)

go_repository(
    name = "com_github_pmezard_go_difflib",
    importpath = "github.com/pmezard/go-difflib",
    sum = "h1:4DBwDE0NGyQoBHbLQYPwSUPoCMWR5BEzIk/f1lZbAQM=",
    version = "v1.0.0",
)

go_repository(
    name = "com_github_shurcool_sanitized_anchor_name",
    importpath = "github.com/shurcooL/sanitized_anchor_name",
    sum = "h1:PdmoCO6wvbs+7yrJyMORt4/BmY5IYyJwS/kOiWx8mHo=",
    version = "v1.0.0",
)

go_repository(
    name = "in_gopkg_check_v1",
    importpath = "gopkg.in/check.v1",
    sum = "h1:yhCVgyC4o1eVCa2tZl7eS0r+SDo693bJlVdllGtEeKM=",
    version = "v0.0.0-20161208181325-20d25e280405",
)

go_repository(
    name = "in_gopkg_yaml_v2",
    importpath = "gopkg.in/yaml.v2",
    sum = "h1:ZCJp+EgiOT7lHqUV2J862kp8Qj64Jo6az82+3Td9dZw=",
    version = "v2.2.2",
)

go_rules_dependencies()

go_register_toolchains(version = "1.18.3")

gazelle_dependencies()

http_file(
    name = "chromiumos_sdk",
    sha256 = "af5d6f53a52bd35ce6513dd0b2bbb78154b71f3b47eb01851e364fae76ef55ac",
    urls = ["https://commondatastorage.googleapis.com/chromiumos-sdk/cros-sdk-2022.07.30.133332.tar.xz"],
)

http_file(
    name = "baselayout_2_2",
    sha256 = "11d4a223b06da545c3e59e07c9195570f334b5b1be05d995df0ebc8ea2203e98",
    urls = ["https://commondatastorage.googleapis.com/chromeos-mirror/gentoo/distfiles/baselayout-2.2.tar.bz2"],
)

http_file(
    name = "bzip2_1_0_8",
    sha256 = "ab5a03176ee106d3f0fa90e381da478ddae405918153cca248e682cd0c4a2269",
    urls = ["https://commondatastorage.googleapis.com/chromeos-mirror/gentoo/distfiles/bzip2-1.0.8.tar.gz"],
)

http_file(
    name = "dbus_1_12_20",
    sha256 = "f77620140ecb4cdc67f37fb444f8a6bea70b5b6461f12f1cbe2cec60fa7de5fe",
    urls = ["https://commondatastorage.googleapis.com/chromeos-mirror/gentoo/distfiles/dbus-1.12.20.tar.gz"],
)

http_file(
    name = "ethtool_4_13",
    sha256 = "b7c1a380007d30eaf261a63b3cfc000f9d93f9eb7626dcd48b5d2a733af99cba",
    urls = ["https://commondatastorage.googleapis.com/chromeos-mirror/gentoo/distfiles/ethtool-4.13.tar.xz"],
)

http_file(
    name = "elt-patches_20210924",
    sha256 = "1b804bbaa7ebc89126762ef639b266f2c105c77f5b6eb6b76f46c77924773268",
    urls = ["https://commondatastorage.googleapis.com/chromeos-mirror/gentoo/distfiles/elt-patches-20210924.tar.xz"],
)

http_file(
    name = "expat_2_4_8",
    sha256 = "f79b8f904b749e3e0d20afeadecf8249c55b2e32d4ebb089ae378df479dcaf25",
    urls = ["https://commondatastorage.googleapis.com/chromeos-mirror/gentoo/distfiles/expat-2.4.8.tar.xz"],
)

http_file(
    name = "fakeroot_1_27",
    sha256 = "3c45eb2d1802a2762069e2e9d21bdd6fb533592bc0cda74c9aff066ab01caddc",
    urls = ["https://commondatastorage.googleapis.com/chromeos-mirror/gentoo/distfiles/fakeroot_1.27.orig.tar.gz"],
)

http_file(
    name = "fuse_2_9_8",
    sha256 = "5e84f81d8dd527ea74f39b6bc001c874c02bad6871d7a9b0c14efb57430eafe3",
    urls = ["https://commondatastorage.googleapis.com/chromeos-mirror/gentoo/distfiles/fuse-2.9.8.tar.gz"],
)

http_file(
    name = "fuse_3_10_4",
    sha256 = "9365b74fd8471caecdb3cc5adf25a821f70a931317ee9103d15bd39089e3590d",
    urls = ["https://commondatastorage.googleapis.com/chromeos-mirror/gentoo/distfiles/fuse-3.10.4.tar.xz"],
)

http_file(
    name = "gmp_6_2_1",
    urls = ["https://commondatastorage.googleapis.com/chromeos-mirror/gentoo/distfiles/gmp-6.2.1.tar.xz"],
)

http_file(
    name = "gmp_6_2_1_man",
    urls = ["https://commondatastorage.googleapis.com/chromeos-mirror/gentoo/distfiles/gmp-man-6.2.1.pdf"],
)

http_file(
    name = "gmp_6_2_1_patch",
    urls = ["https://commondatastorage.googleapis.com/chromeos-mirror/gentoo/distfiles/gmp-6.2.1-arm64-darwin.patch.bz2"],
)

http_file(
    name = "mpfr_3_1_3",
    urls = ["https://commondatastorage.googleapis.com/chromeos-mirror/gentoo/distfiles/mpfr-3.1.3.tar.xz"],
)

http_file(
    name = "ncurses_5_9",
    sha256 = "9046298fb440324c9d4135ecea7879ffed8546dd1b58e59430ea07a4633f563b",
    urls = ["https://commondatastorage.googleapis.com/chromeos-mirror/gentoo/distfiles/ncurses-5.9.tar.gz"],
)

http_file(
    name = "pcre_8_44",
    sha256 = "19108658b23b3ec5058edc9f66ac545ea19f9537234be1ec62b714c84399366d",
    urls = ["https://commondatastorage.googleapis.com/chromeos-mirror/gentoo/distfiles/pcre-8.44.tar.bz2"],
)

http_file(
    name = "readline_6_3_p8_r3",
    sha256 = "2dc5e5d1178f01baf2481c8364fedcf7c32df2cd647f496001d06e5a6a7c8dd1",
    urls = ["https://commondatastorage.googleapis.com/chromeos-localmirror/distfiles/readline-6.3_p8-r3.tbz2"],
)

http_file(
    name = "readline_8_1",
    sha256 = "f8ceb4ee131e3232226a17f51b164afc46cd0b9e6cef344be87c65962cb82b02",
    urls = ["https://commondatastorage.googleapis.com/chromeos-mirror/gentoo/distfiles/readline-8.1.tar.gz"],
)

http_file(
    name = "readline_8_1_patch_001",
    sha256 = "682a465a68633650565c43d59f0b8cdf149c13a874682d3c20cb4af6709b9144",
    urls = ["https://commondatastorage.googleapis.com/chromeos-mirror/gentoo/distfiles/readline81-001"],
)

http_file(
    name = "readline_8_1_patch_002",
    sha256 = "e55be055a68cb0719b0ccb5edc9a74edcc1d1f689e8a501525b3bc5ebad325dc",
    urls = ["https://commondatastorage.googleapis.com/chromeos-mirror/gentoo/distfiles/readline81-002"],
)

http_file(
    name = "sqlite_src_3320300",
    sha256 = "9312f0865d3692384d466048f746d18f88e7ffd1758b77d4f07904e03ed5f5b9",
    urls = ["https://commondatastorage.googleapis.com/chromeos-mirror/gentoo/distfiles/sqlite-src-3320300.zip"],
)

http_file(
    name = "sqlite_doc_3320300",
    sha256 = "36920536daf7f8b19c2e646dc79db62e13cc1a992f562ba9a11fa7c191f24a4e",
    urls = ["https://commondatastorage.googleapis.com/chromeos-mirror/gentoo/distfiles/sqlite-doc-3320300.zip"],
)

http_file(
    name = "zlib_1_2_11",
    sha256 = "c3e5e9fdd5004dcb542feda5ee4f0ff0744628baf8ed2dd5d66f8ca1197cb1a1",
    urls = ["https://commondatastorage.googleapis.com/chromeos-mirror/gentoo/distfiles/zlib-1.2.11.tar.gz"],
)

http_file(
    name = "zlib_1_2_11_cygwin_gzopen_w_patch",
    sha256 = "2eea64808bc6edd2f12a7f7ef66381a73a546fa31ec9f95e5305cf51f3db3d86",
    urls = ["https://commondatastorage.googleapis.com/chromeos-mirror/gentoo/distfiles/zlib-1.2.11-cygwin-gzopen_w.patch"],
)

http_file(
    name = "zlib_1_2_7_cygwin_minizip_patch",
    sha256 = "0352e8b84ea4c9c8e3de1817fe42db6a729cce834d5784b399974418ec0e44e8",
    urls = ["https://commondatastorage.googleapis.com/chromeos-mirror/gentoo/distfiles/zlib-1.2.7-cygwin-minizip.patch"],
)

http_file(
    name = "dumb_init",
    executable = True,
    sha256 = "e874b55f3279ca41415d290c512a7ba9d08f98041b28ae7c2acb19a545f1c4df",
    urls = ["https://github.com/Yelp/dumb-init/releases/download/v1.2.5/dumb-init_1.2.5_x86_64"],
)

http_file(
    name = "arm64_generic_linux_headers_4_14_r52",
    downloaded_file_path = "linux-headers-4.14-r52.tbz2",
    sha256 = "e9d881c74ddfd6243866460506a9859919a3c1be7e1dc9b5777492f6a9ca03cf",
    urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/board/arm64-generic/postsubmit-R106-14995.0.0-37502-8807992176298868113/packages/sys-kernel/linux-headers-4.14-r52.tbz2"],
)

http_file(
    name = "arm64_generic_gcc_libs_10_2_0_r4",
    downloaded_file_path = "gcc-libs-10.2.0-r4.tbz2",
    sha256 = "21dd868049fbd44bb4c6f4657b1f23f1a6e3879d72678bf6178ed580f4f8c8a8",
    urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/board/arm64-generic/postsubmit-R106-14995.0.0-37502-8807992176298868113/packages/sys-libs/gcc-libs-10.2.0-r4.tbz2"],
)

http_file(
    name = "arm64_generic_libcxx_15_0_pre458507_r5",
    downloaded_file_path = "libcxx-15.0_pre458507-r5.tbz2",
    sha256 = "f1f0c27fbedd9f6476d796350fd47c07f54f79660f395cbaa14ea2ffdfc0412e",
    urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/board/arm64-generic/postsubmit-R106-14995.0.0-37502-8807992176298868113/packages/sys-libs/libcxx-15.0_pre458507-r5.tbz2"],
)

http_file(
    name = "arm64_generic_llvm_libunwind_15_0_pre458507_r4",
    downloaded_file_path = "llvm-libunwind-15.0_pre458507-r4.tbz2",
    sha256 = "9f21b16420f77ae5e1e377ba17b4cdb6457e4395c52c8e06fa421c6873f82a94",
    urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/board/arm64-generic/postsubmit-R106-14995.0.0-37502-8807992176298868113/packages/sys-libs/llvm-libunwind-15.0_pre458507-r4.tbz2"],
)

http_file(
    name = "amd64_host_binutils_2_36_1_r8",
    downloaded_file_path = "binutils-2.36.1-r8.tbz2",
    urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2022.07.20.214841/packages/cross-aarch64-cros-linux-gnu/binutils-2.36.1-r8.tbz2"],
)

http_file(
    name = "amd64_host_compiler_rt_15_0_pre458507_r6",
    downloaded_file_path = "compiler-rt-15.0_pre458507-r6.tbz2",
    urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2022.07.20.214841/packages/cross-aarch64-cros-linux-gnu/compiler-rt-15.0_pre458507-r6.tbz2"],
)

http_file(
    name = "amd64_host_gcc_10_2_0_r28",
    downloaded_file_path = "gcc-10.2.0-r28.tbz2",
    urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2022.07.20.214841/packages/cross-aarch64-cros-linux-gnu/gcc-10.2.0-r28.tbz2"],
)

http_file(
    name = "amd64_host_gdb_9_2_20200923_r9",
    downloaded_file_path = "gdb-9.2.20200923-r9.tbz2",
    urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2022.07.20.214841/packages/cross-aarch64-cros-linux-gnu/gdb-9.2.20200923-r9.tbz2"],
)

http_file(
    name = "amd64_host_glibc_2_33_r17",
    downloaded_file_path = "glibc-2.33-r17.tbz2",
    urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2022.07.20.214841/packages/cross-aarch64-cros-linux-gnu/glibc-2.33-r17.tbz2"],
)

http_file(
    name = "amd64_host_go_1_18_r1",
    downloaded_file_path = "go-1.18-r1.tbz2",
    urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2022.07.20.214841/packages/cross-aarch64-cros-linux-gnu/go-1.18-r1.tbz2"],
)

http_file(
    name = "amd64_host_libcxx_15_0_pre458507_r5",
    downloaded_file_path = "libcxx-15.0_pre458507-r5.tbz2",
    urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2022.07.20.214841/packages/cross-aarch64-cros-linux-gnu/libcxx-15.0_pre458507-r5.tbz2"],
)

http_file(
    name = "amd64_host_libxcrypt_4_4_28_r1",
    downloaded_file_path = "libxcrypt-4.4.28-r1.tbz2",
    urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2022.07.20.214841/packages/cross-aarch64-cros-linux-gnu/libxcrypt-4.4.28-r1.tbz2"],
)

http_file(
    name = "amd64_host_linux_headers_4_14_r52",
    downloaded_file_path = "linux-headers-4.14-r52.tbz2",
    urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2022.07.20.214841/packages/cross-aarch64-cros-linux-gnu/linux-headers-4.14-r52.tbz2"],
)

http_file(
    name = "amd64_host_llvm_libunwind_15_0_pre458507_r4",
    downloaded_file_path = "llvm-libunwind-15.0_pre458507-r4.tbz2",
    urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2022.07.20.214841/packages/cross-aarch64-cros-linux-gnu/llvm-libunwind-15.0_pre458507-r4.tbz2"],
)
