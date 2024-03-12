# Copyright 2024 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""A binary to expand jinja2 templates."""

import json
import pathlib
import sys

from jinja2 import DictLoader
from jinja2 import Environment


_TEMPLATE_NAME = "default"


class TemplateError(Exception):
    """An exception raised by a template"""


def template_error(fmt, **kwargs):
    raise TemplateError(fmt.format(**kwargs))


def main(template: pathlib.Path, vars_file: pathlib.Path, out: pathlib.Path):
    vars_values = json.loads(vars_file.read_text())
    template = template.read_text()

    env = Environment(
        loader=DictLoader({_TEMPLATE_NAME: template}),
        # This isn't HTML. We don't need HTML autoescaping.
        autoescape=False,
        lstrip_blocks=True,
        trim_blocks=True,
    )
    env.globals["error"] = template_error
    template = env.get_template(_TEMPLATE_NAME)
    out.write_text(template.render(**vars_values))


if __name__ == "__main__":
    main(
        template=pathlib.Path(sys.argv[1]),
        vars_file=pathlib.Path(sys.argv[2]),
        out=pathlib.Path(sys.argv[3]),
    )
