load("//rules_cros/third_party:github_archive.bzl", "github_archive")

VERSION = "0.1.103"
CHECKSUM = "bba530fe435d8f9195a32c295147677c58b060e2c63d2d4204ed8a6c9621d0dd"

# Squashfuse is built with a patch to respond ioctl requests with ENOTTY. This
# is needed for overlayfs to work with squashfuse. See the following thread for
# details:
# https://lore.kernel.org/lkml/4B9D76D5-C794-4A49-A76F-3D4C10385EE0@kohlschutter.com/T/

def squashfuse_repositories():
    github_archive(
        name = "squashfuse",
        build_file = Label("//rules_cros/third_party:BUILD.all_srcs.bazel"),
        checksum = CHECKSUM,
        github_user = "vasi",
        github_repo = "squashfuse",
        tag = VERSION,
        patches = [
            "//rules_cros/third_party/squashfuse:patches/ioctl.patch",
        ],
    )
