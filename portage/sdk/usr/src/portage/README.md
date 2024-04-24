This directory contains bazel specific patches to portage.  We generally add
things in here for a couple reasons:
1) The patch is very bazel specific and we don't want regular portage users
   being affected.
2) We want a patch to take effect without having to repin the stage1 SDK.
3) The patches are still experimental and haven't been accepted upstream.

Patches can be removed once they have landed in out portage fork and the
stage1 SDK has been repinned.

Patches are generated from the ~/chromiumos/src/third_party/portage_tool
directory. You can find all the existing patches using this gerrit query:
https://chromium-review.googlesource.com/q/topic:%22bazel-portage-patches%22

When generating new patches or modifying existing patches, please make sure to
check out all the outstanding patches and regenerate the whole chain. This makes
it easier to stack changes and avoid merge conflicts.

You can use the following commands to generate the patches:

    $ cd src/third_party/portage_tool
    $ git format-patch m/main
    $ rm -rf ../../bazel/portage/sdk/usr/src/portage/*.patch
    $ mv *.patch ../../bazel/portage/sdk/usr/src/portage/
    $ ../../bazel/portage/sdk/usr/src/portage/fix-paths.sh
