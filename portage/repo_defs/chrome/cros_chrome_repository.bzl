# Copyright 2022 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("@bazel_skylib//lib:paths.bzl", "paths")

def _exec(ctx, cmd, msg = None, retries = 0, **kwargs):
    env = dict(ctx.os.environ)
    env.update(kwargs)
    if msg:
        ctx.report_progress(msg)

    st = None
    for attempt in range(0, retries + 1):
        # Use 3600 as timeout because gclient can take a long time to finish.
        st = ctx.execute(cmd, timeout = 3600, environment = env)
        if st.return_code:
            if attempt == retries:
                fail("Error running attempt %s/%s for command %s:\n%s%s" %
                     (attempt + 1, retries + 1, cmd, st.stdout, st.stderr))
            else:
                print("Error running attempt %s/%s for command %s:\n%s%s\nRetrying." %
                      (attempt + 1, retries + 1, cmd, st.stdout, st.stderr))
        else:
            print("Finished running command %s (attempt %s/%s)" % (cmd, attempt + 1, retries + 1))
            break
    return st.stdout

def _git(ctx, repo, args, msg = None):
    cmd = ["git", "-C", repo] + args
    return _exec(ctx, cmd, msg, retries = 1)

def _exec_with_gce_context_if_needed(ctx, cmd, msg = None, retries = 0, **kwargs):
    """Runs the specified command in a luci context which uses the GCE metadata host for authentication if needed"""
    wrapper = []
    if ctx.os.environ.get("GCE_METADATA_HOST"):
        wrapper = ["luci-auth", "context", "-service-account-json", ":gce", "--"]
    _exec(ctx, wrapper + cmd, msg, **kwargs)

def _cros_chrome_repository_impl(ctx):
    """Repository rule that downloads the Chromium/Chrome source."""

    print("Started fetching Chromium source.")

    tar = ctx.which("tar")
    if not tar:
        fail("tar was not found on the path")

    # TODO(b/342064824): Stop using `pzstd` found on the system.
    pzstd = ctx.which("pzstd")
    if not pzstd:
        fail("pzstd was not found on the path")

    ctx.template(".gclient", ctx.attr._gclient_template, {
        "{internal}": str(ctx.attr.internal),
        "{revision}": ctx.attr.revision,
    })

    ctx.delete("src")
    _exec(ctx, ["git", "init", "src"])
    _git(ctx, "src", ["remote", "add", "origin", ctx.attr.remote])
    _git(ctx, "src", ["fetch", "--depth=1", "origin", ctx.attr.revision], "Fetching " + ctx.attr.revision)
    _git(ctx, "src", ["reset", "--hard", ctx.attr.revision], "Resetting to " + ctx.attr.revision)
    _git(ctx, "src", ["clean", "-xdf"])

    # The chromium repo is huge and gclient will perform a blind `git fetch`
    # that attempts to fetch all the refs. We want to ensure we only pull
    # the specific tag, so we override the fetch config
    _git(
        ctx,
        "src",
        [
            "config",
            "remote.origin.fetch",
            ctx.attr.revision,
        ],
    )

    pwd = _exec(ctx, ["pwd"]).strip()

    # Set the cache directories so we don't write to the users home directory.
    cipd_cache_dir = paths.join(pwd, ".cipd-cache")
    vpython_root = paths.join(pwd, ".vpython-root")
    depot_tools_path = ctx.workspace_root.get_child("chromium/depot_tools")

    _exec_with_gce_context_if_needed(
        ctx,
        [
            depot_tools_path.get_child("gclient"),
            "sync",
            "--noprehooks",
            "--nohooks",
            # This unfortunately doesn't change how much data we fetch.
            "--no-history",
            # This still pulls in 10000 commits. I wish we could specify the depth.
            "--shallow",
            "--jobs",
            "12",
        ],
        "Fetching third_party chromium dependencies",
        PATH = "{}:{}".format(depot_tools_path, ctx.os.environ["PATH"]),
        CIPD_CACHE_DIR = cipd_cache_dir,
        VPYTHON_VIRTUALENV_ROOT = vpython_root,
        DEPOT_TOOLS_UPDATE = "0",
    )

    # When running hooks `update_depot_tools_toggle.py` will write this file
    # with a timestamp. By writing it ourselves we can keep the tarball
    # reproducible.
    ctx.file(paths.join(pwd, "src/third_party/depot_tools/.disable_auto_update"), "")
    if ctx.attr.internal:
        ctx.file(paths.join(pwd, "src/third_party/devtools-frontend-internal/third_party/depot_tools/.disable_auto_update"), "")

    # This command will populate the cipd cache and create a python venv.
    _exec_with_gce_context_if_needed(
        ctx,
        [depot_tools_path.get_child("ensure_bootstrap")],
        "Downloading depot_tools dependencies",
        PATH = "{}:{}".format(depot_tools_path, ctx.os.environ["PATH"]),
        CIPD_CACHE_DIR = cipd_cache_dir,
        VPYTHON_VIRTUALENV_ROOT = vpython_root,
        DEPOT_TOOLS_UPDATE = "0",
    )

    # Run the hooks with the depot tools pinned by chromium.
    _exec_with_gce_context_if_needed(
        ctx,
        [
            depot_tools_path.get_child("gclient"),
            "runhooks",
            "--force",
            "--jobs",
            "12",
        ],
        "Running chromium hooks",
        PATH = "{}:{}".format(depot_tools_path, ctx.os.environ["PATH"]),
        CIPD_CACHE_DIR = cipd_cache_dir,
        VPYTHON_VIRTUALENV_ROOT = vpython_root,
        DEPOT_TOOLS_UPDATE = "0",
    )

    # See https://reproducible-builds.org/docs/archives/
    tar_common = [
        "tar",
        "--format",
        "gnu",
        "--sort",
        "name",
        "--mtime",
        "1970-1-1 00:00Z",
        "--owner",
        "0",
        "--group",
        "0",
        "--numeric-owner",
        "--exclude-vcs",
        "-I{}".format(pzstd),
        "--create",
        # r: Apply transformation to regular archive members.
        # S: Do not apply transformation to symbolic link targets.
        # h: Apply transformation to hard link targets.
        "--transform=flags=rSh;s,^,/home/root/chrome_root/,",
    ]

    # Remove CIPD files to make this hermetic.
    ctx.delete("src/third_party/depot_tools/.cipd_bin")

    # Remove CIPD cache files in src/third_party/depot_tools/bootstrap-*_bin/.cipd
    for path in _exec(
        ctx,
        [
            "find",
            "src/third_party/depot_tools",
            "-maxdepth",
            "1",
            "-type",
            "d",
            "-name",
            "bootstrap-*_bin",
        ],
    ).strip("\n").split("\n"):
        ctx.delete(paths.join(path, ".cipd"))

    # Remove depot_tools/metrics.cfg because its contents can be different for
    # each user and it's unecessary.
    ctx.delete("src/third_party/depot_tools/metrics.cfg")

    # Remove reclient_cfgs/reproxy.cfg because its contents contains a full path
    # and it's unnecessary.
    ctx.delete("src/buildtools/reclient_cfgs/reproxy.cfg")

    # We use zstd since it's way faster than gzip and should be installed by
    # default on most distributions. Hopefully the compression algorithm doesn't
    # change between hosts, otherwise the output won't be hermetic.
    #
    # The compressed src should be around ~5GiB, uncompressed it's about 20 GiB.
    _exec(
        ctx,
        tar_common + [
            "--file",
            "src.tar.zst",
            # chromium src and depot_tools with it's own .cipd_bin
            "src",
            # Needed to signal the gclient root
            ".gclient",
            # Hashes of all the dependencies. Useful since we don't include the
            # .git directories.
            ".gclient_entries",
            # We don't include the .vpython-root since it contains absolute
            # symlinks which we can't use inside the chroot. Since we have the
            # cache and pkgs we can recreate it in the chroot without network
            # access.
        ],
        ZSTD_NBTHREADS = "0",
        msg = "Tarring up Chromium src",
    )

    _exec(
        ctx,
        tar_common + [
            "--file",
            "cipd-cache.tar.zst",
            ".cipd-cache",
        ],
        ZSTD_NBTHREADS = "0",
        msg = "Tarring up CIPD cache files",
    )

    ctx.delete(".cipd")
    ctx.delete(".cipd-cache")
    ctx.delete(".gclient")
    ctx.delete(".gclient_entries")
    ctx.delete(".gclient_previous_custom_vars")
    ctx.delete(".gclient_previous_sync_commits")
    ctx.delete(".vpython-root")
    ctx.delete("src")

    ctx.file("WORKSPACE", "workspace(name = \"{name}\")\n".format(name = ctx.name))
    ctx.template("BUILD.bazel", ctx.attr._build_file)

_cros_sdk_repository_attrs = {
    "internal": attr.bool(
        doc = """If true download Chrome if false download Chromium.""",
        mandatory = True,
    ),
    "remote": attr.string(
        doc = "The URI of the remote Chromium Git repository",
        default = "https://chromium.googlesource.com/chromium/src.git",
    ),
    "revision": attr.string(
        doc = """The expected revision of the file downloaded.""",
        mandatory = True,
    ),
    "_build_file": attr.label(
        allow_single_file = True,
        default = ":BUILD.chrome-template",
    ),
    "_gclient_template": attr.label(
        doc = """.gclient template.""",
        default = ":gclient-template.py",
    ),
}

cros_chrome_repository = repository_rule(
    implementation = _cros_chrome_repository_impl,
    attrs = _cros_sdk_repository_attrs,
)
