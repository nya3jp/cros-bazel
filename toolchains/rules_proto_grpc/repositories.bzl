load("//rules_cros/third_party:github_archive.bzl", "github_archive")

VERSION = "4.2.0"
CHECKSUM = "bbe4db93499f5c9414926e46f9e35016999a4e9f6e3522482d3760dc61011070"

def rules_proto_grpc_repositories():
    github_archive(
        name = "rules_proto_grpc",
        checksum = CHECKSUM,
        github_repo = "rules_proto_grpc",
        github_user = "rules-proto-grpc",
        tag = VERSION,
    )
