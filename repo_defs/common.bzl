# Copyright 2026 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

REPO_AUTH_ENVIRON = [
    "GCE_METADATA_HOST",
    "LUCI_CONTEXT",
    "HOME",
]

def exec(ctx, cmd, msg = None, retries = 0, delay = 60, **kwargs):
    env = dict(ctx.os.environ)
    env.update(kwargs)
    if msg:
        ctx.report_progress(msg)

    st = None
    for attempt in range(0, retries + 1):
        # Use 3600 as timeout because gclient and git can take a long time to finish.
        st = ctx.execute(cmd, timeout = 3600, environment = env)
        if st.return_code:
            if attempt == retries:
                fail("Error running attempt %s/%s for command %s:\n%s%s" %
                     (attempt + 1, retries + 1, cmd, st.stdout, st.stderr))
            else:
                print("Error running attempt %s/%s for command %s:\n%s%s\nRetrying in %s s." %
                      (attempt + 1, retries + 1, cmd, st.stdout, st.stderr, delay))

                # Ignore the return code since we don't want to fail if sleep
                # fails for some reason.
                ctx.execute(["sleep", str(delay)])
        else:
            print("Finished running command %s (attempt %s/%s)" % (cmd, attempt + 1, retries + 1))
            break
    return st.stdout

def git(ctx, repo, args, msg = None):
    cmd = ["git", "-C", repo] + args
    return exec(ctx, cmd, msg, retries = 1)

def exec_with_gce_context_if_needed(ctx, cmd, msg = None, retries = 0, delay = 60, **kwargs):
    """Runs the specified command in a luci context which uses the GCE metadata host for authentication if needed."""
    wrapper = []
    has_gce = bool(ctx.os.environ.get("GCE_METADATA_HOST"))
    has_luci_ctx = bool(ctx.os.environ.get("LUCI_CONTEXT"))

    if has_luci_ctx:
        print("_exec_with_gce_context_if_needed: Wrapping in luci-auth context (LUCI_CONTEXT present).")
        wrapper = ["luci-auth", "context", "-scopes-context", "--"]
    elif has_gce:
        print("_exec_with_gce_context_if_needed: Wrapping in luci-auth context (GCE_METADATA_HOST present, LUCI_CONTEXT missing).")
        wrapper = ["luci-auth", "context", "-scopes-context", "-service-account-json", ":gce", "--"]
    else:
        print("_exec_with_gce_context_if_needed: Using ambient environment (GCE_METADATA_HOST=%s, LUCI_CONTEXT=%s)." %
              (has_gce, has_luci_ctx))

    return exec(ctx, wrapper + cmd, msg, retries = retries, delay = delay, **kwargs)
