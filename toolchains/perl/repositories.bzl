load("//rules_cros/third_party:github_archive.bzl", "github_archive")

RULES_VERSION = "0.1.0"
CHECKSUM = "5cefadbf2a49bf3421ede009f2c5a2c9836abae792620ed2ff99184133755325"

def perl_repositories():
    github_archive(
        name = "rules_perl",
        checksum = CHECKSUM,
        github_user = "bazelbuild",
        github_repo = "rules_perl",
        tag = RULES_VERSION,
    )
