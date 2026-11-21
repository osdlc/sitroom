#!/bin/sh
# Runs inside a distro container from the release workflow's linux-test job,
# in a folder holding packages/ (the .deb, .rpm and .apk files), release/
# (the Linux archives and SHA256SUMS under v<version>/) and site/.
#
# Installs the package with the distro's own tool, runs both programs,
# removes it, then runs install.sh against release/ and checks it chose the
# build this distro needs.
#
# Usage: scripts/linux-packages-test.sh <version> <gnu|musl>
set -eu

version="$1"
want_libc="$2"

case "$(uname -m)" in
  x86_64) deb=amd64 rpm=x86_64 apk=x86_64 ;;
  aarch64) deb=arm64 rpm=aarch64 apk=aarch64 ;;
  *) echo "no package for $(uname -m)" >&2; exit 1 ;;
esac

if command -v apt-get >/dev/null 2>&1; then
  export DEBIAN_FRONTEND=noninteractive
  apt-get update -qq
  apt-get install -y -qq curl ca-certificates >/dev/null
  apt-get install -y "./packages/sitroom_${version}_${deb}.deb"
  remove() { apt-get remove -y sitroom; }
elif command -v dnf >/dev/null 2>&1; then
  dnf install -y "./packages/sitroom-${version}-1.${rpm}.rpm"
  remove() { dnf remove -y sitroom; }
elif command -v apk >/dev/null 2>&1; then
  apk add --no-cache curl
  apk add --allow-untrusted "./packages/sitroom_${version}_${apk}.apk"
  remove() { apk del sitroom; }
else
  echo 'no apt-get, dnf or apk here' >&2
  exit 1
fi

echo '--- the package'
# The files, then that they are on PATH. Not `command -v` = /usr/bin/...:
# on Fedora /usr/sbin is a link to /usr/bin and comes first on PATH.
test -x /usr/bin/sitroom
test -x /usr/bin/sit
command -v sitroom >/dev/null
command -v sit >/dev/null
# Ubuntu's container images tell dpkg to leave out /usr/share/doc.
if ! grep -qs '^path-exclude=/usr/share/doc' /etc/dpkg/dpkg.cfg.d/*; then
  test -f /usr/share/doc/sitroom/README.md
fi
sitroom --version
sit --version
remove
# The files, not `command -v`: the shell remembers where it found them.
if [ -e /usr/bin/sitroom ] || [ -e /usr/bin/sit ]; then
  echo 'sitroom is still installed after removing the package' >&2
  exit 1
fi

echo '--- install.sh'
log="$(mktemp)"
SITROOM_DOWNLOAD_BASE="file://$PWD/release" \
  sh site/public/install.sh --version "$version" --to /tmp/bin >"$log" 2>&1 || {
  cat "$log"
  exit 1
}
cat "$log"
grep -q "sitroom-${version}-$(uname -m)-unknown-linux-${want_libc}.tar.gz" "$log" || {
  echo "install.sh didn't choose the ${want_libc} build" >&2
  exit 1
}
PATH="/tmp/bin:$PATH"
sitroom --version
sit --version
