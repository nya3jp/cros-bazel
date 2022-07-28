EAPI=7
KEYWORDS="*"
SLOT="0"

S="${WORKDIR}"

src_compile() {
  echo <<EOF > hello
#!/bin/bash
echo "Hello, world!"
EOF
  chmod +x hello
}

src_install() {
  exeinto /usr/bin
  doexe hello
}
