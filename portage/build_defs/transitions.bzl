# Copyright 2024 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

# We need to lead the private one, since the public one is a macro.
load("@rules_go//go/private/rules:binary.bzl", "go_binary")
load("@rules_pkg//pkg/private/tar:tar.bzl", "pkg_tar_impl")
load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_test")

visibility("public")

def _rule_impl(ctx):
    return ctx.super()

def _transition_to(flags):
    def transition_impl(settings, attr):
        return flags

    return transition(
        implementation = transition_impl,
        inputs = [],
        outputs = flags.keys(),
    )

def _rule_with_flags(parent, flags):
    return rule(
        implementation = _rule_impl,
        parent = parent,
        cfg = _transition_to(flags),
    )

_ENSURE_CACHE_HIT = {
    "//command_line_option:compilation_mode": "opt",
    "//command_line_option:stamp": False,
}

# This transition is used to ensure that changing --compilation_mode doesn't
# invalidate the cache for every single ebuild, and ensures that you can still
# get a *remote* cache hit even if you change flags.
rust_for_ebuild_binary = _rule_with_flags(
    rust_binary,
    _ENSURE_CACHE_HIT,
)

rust_for_ebuild_test = _rule_with_flags(
    rust_test,
    _ENSURE_CACHE_HIT,
)

go_for_ebuild_binary = _rule_with_flags(
    go_binary,
    _ENSURE_CACHE_HIT,
)

# Unfortunately the pkg_tar rule is private, and only the macro is public.
# Thus, we have to reimplement the macro.
_pkg_tar_for_ebuild = _rule_with_flags(
    pkg_tar_impl,
    _ENSURE_CACHE_HIT,
)

def pkg_tar_for_ebuild(name, **kwargs):
    extension = kwargs.get("extension") or "tar"
    if extension[0] == ".":
        extension = extension[1:]
    _pkg_tar_for_ebuild(
        name = name,
        out = kwargs.pop("out", None) or (name + "." + extension),
        **kwargs
    )
