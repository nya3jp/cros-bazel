load("//bazel/third_party:github_archive.bzl", "github_archive")

RULES_VERSION = "0.14.0"
CHECKSUM = "2625a71dafa42fb63348bcb04498cb9e83be6ea0e99e0359863345dc1cfd65fb"

def rust_repositories():
    github_archive(
        name = "rules_rust",
        checksum = CHECKSUM,
        github_user = "bazelbuild",
        github_repo = "rules_rust",
        tag = RULES_VERSION,
    )
