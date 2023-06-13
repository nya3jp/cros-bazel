# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")
load("@//bazel:repo/repo_repository.bzl", "repo_repository")
load("@//bazel:repo/cipd.bzl", "cipd_file")
load("@//bazel/chrome:cros_chrome_repository.bzl", "cros_chrome_repository")

def portage_dependencies():
    {% for dist in dists -%}
    {% set url = dist.urls[0] %}
    {% if url is starting_with("cipd") %}
    cipd_file(
        name = "{{ dist.repository_name }}",
        downloaded_file_path = "{{ dist.filename }}",
        url = "{{ url }}"
    )
    {% else %}
    http_file(
        name = "{{ dist.repository_name }}",
        downloaded_file_path = "{{ dist.filename }}",
        integrity = "{{ dist.integrity }}",
        urls = [
            {%- for url in dist.urls %}
            "{{ url }}",
            {%- endfor %}
        ],
    )
    {% endif %}
    {% endfor %}

    {% for repo in repos -%}
    repo_repository(
        name = "{{ repo.name }}",
        project = "{{ repo.project }}",
        tree = "{{ repo.tree_hash }}",
    )
    {% endfor %}

    {% for version in chrome -%}
    cros_chrome_repository(
        name = "chrome-{{ version }}",
        tag = "{{ version }}",
        gclient = "@depot_tools//:gclient.wrapper.sh"
    )
    {% endfor %}
