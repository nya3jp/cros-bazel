load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")

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
