load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")
load("@//bazel:repo/repo_repository.bzl", "repo_repository")
load("@//bazel/chrome:cros_chrome_repository.bzl", "cros_chrome_repository")

def portage_dependencies():
    {% for dist in dists -%}
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
