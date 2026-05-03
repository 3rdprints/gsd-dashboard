#!/bin/sh
if (set -o pipefail) 2>/dev/null; then
  set -euo pipefail
else
  set -eu
fi

yes_flag=false

for arg in "$@"; do
  case "$arg" in
    --yes)
      yes_flag=true
      ;;
    --help|-h)
      printf '%s\n' "Usage: install.sh [--yes]"
      printf '%s\n' "Downloads the native GSD Dashboard installer for this OS and architecture."
      printf '%s\n' "Verification: downloads CHECKSUM_URL when set, otherwise <artifact URL>.sha256."
      exit 0
      ;;
    *)
      printf '%s\n' "Unknown option: $arg" >&2
      exit 1
      ;;
  esac
done

base_url="${GSD_DASHBOARD_BASE_URL:-https://smacdonald.github.io/gsd-dashboard/}"
base_url="${base_url%/}/"
manual_url="${base_url}#platform-downloads"

raw_os="$(uname -s)"
raw_arch="$(uname -m)"
os=""
arch=""
artifact=""

case "$raw_os" in
  Darwin)
    os="macos"
    ;;
  Linux)
    os="linux"
    ;;
  MINGW*|MSYS*|CYGWIN*)
    os="windows"
    ;;
  *)
    printf '%s\n' "Unsupported OS/arch: $raw_os/$raw_arch" >&2
    printf '%s\n' "Manual downloads: $manual_url" >&2
    exit 1
    ;;
esac

case "$raw_arch" in
  arm64|aarch64)
    arch="arm64"
    ;;
  x86_64|amd64)
    arch="x86_64"
    ;;
  *)
    printf '%s\n' "Unsupported OS/arch: $raw_os/$raw_arch" >&2
    printf '%s\n' "Manual downloads: $manual_url" >&2
    exit 1
    ;;
esac

detect_linux_family() {
  if [ ! -r /etc/os-release ]; then
    printf '%s\n' "appimage"
    return
  fi

  os_release="$(cat /etc/os-release)"
  case "$os_release" in
    *ID_LIKE=*debian*|*ID=debian*|*ID=ubuntu*)
      printf '%s\n' "deb"
      ;;
    *ID_LIKE=*rhel*|*ID_LIKE=*fedora*|*ID_LIKE=*suse*|*ID=fedora*|*ID=rhel*|*ID=centos*|*ID=opensuse*)
      printf '%s\n' "rpm"
      ;;
    *)
      printf '%s\n' "appimage"
      ;;
  esac
}

case "$os/$arch" in
  macos/arm64|macos/x86_64)
    artifact="GSD-Dashboard.dmg"
    ;;
  windows/x86_64)
    artifact="GSD-Dashboard.msi"
    ;;
  linux/x86_64)
    linux_family="$(detect_linux_family)"
    case "$linux_family" in
      deb)
        artifact="gsd-dashboard.deb"
        ;;
      rpm)
        artifact="gsd-dashboard.rpm"
        ;;
      *)
        artifact="gsd-dashboard.AppImage"
        ;;
    esac
    ;;
  *)
    printf '%s\n' "Unsupported OS/arch: $os/$arch" >&2
    printf '%s\n' "Manual downloads: $manual_url" >&2
    exit 1
    ;;
esac

download_url="${base_url}downloads/${artifact}"
checksum_url="${CHECKSUM_URL:-${download_url}.sha256}"
prompt='Install `${artifact}` for `${os}/${arch}`?'

printf '%s\n' "Detected OS/arch: $os/$arch"
printf '%s\n' "Selected artifact: $artifact"
printf '%s\n' "Download URL: $download_url"
printf '%s\n' "Checksum URL: $checksum_url"

if [ "$yes_flag" != "true" ]; then
  if [ ! -r /dev/tty ]; then
    printf '%s\n' "Interactive confirmation requires a terminal. Re-run with --yes for noninteractive installs." >&2
    exit 1
  fi

  printf 'Install `%s` for `%s/%s`? [y/N] ' "$artifact" "$os" "$arch"
  read -r answer </dev/tty
  case "$answer" in
    y|Y|yes|YES)
      ;;
    *)
      printf '%s\n' "Cancelled."
      exit 0
      ;;
  esac
fi

download_dir="${GSD_DASHBOARD_DOWNLOAD_DIR:-${HOME}/Downloads}"
mkdir -p "$download_dir"
download_path="${download_dir}/${artifact}"
checksum_path="${download_path}.sha256"
curl -fsSL --retry 3 --retry-connrefused --retry-delay 2 --retry-max-time 60 --connect-timeout 10 --max-time 300 "$download_url" -o "$download_path"
curl -fsSL --retry 3 --retry-connrefused --retry-delay 2 --retry-max-time 60 --connect-timeout 10 --max-time 60 "$checksum_url" -o "$checksum_path"

verify_checksum() {
  checksum_value="$(awk '{ print $1; exit }' "$checksum_path")"
  if ! printf '%s' "$checksum_value" | grep -Eq '^[0-9a-fA-F]{64}$'; then
    printf '%s\n' "Checksum file did not contain a SHA-256 digest: $checksum_url" >&2
    exit 1
  fi

  if command -v sha256sum >/dev/null 2>&1; then
    printf '%s  %s\n' "$checksum_value" "$download_path" | sha256sum -c -
  elif command -v shasum >/dev/null 2>&1; then
    printf '%s  %s\n' "$checksum_value" "$download_path" | shasum -a 256 -c -
  else
    printf '%s\n' "Neither sha256sum nor shasum is available; cannot verify $artifact." >&2
    exit 1
  fi
}

verify_checksum

case "$artifact" in
  *.dmg)
    open "$download_path"
    printf '%s\n' "Opened $artifact. Drag GSD Dashboard into Applications to finish installation."
    ;;
  *.msi|*.exe)
    if command -v cmd.exe >/dev/null 2>&1; then
      cmd.exe /c start "" "$(cygpath -w "$download_path" 2>/dev/null || printf '%s' "$download_path")"
    else
      printf '%s\n' "Downloaded $artifact to $download_path. Open it to finish installation."
    fi
    ;;
  *.deb)
    sudo dpkg -i "$download_path"
    ;;
  *.rpm)
    sudo rpm -Uvh "$download_path"
    ;;
  *.AppImage)
    install_dir="${HOME}/Applications"
    mkdir -p "$install_dir"
    chmod +x "$download_path"
    mv "$download_path" "${install_dir}/GSD-Dashboard.AppImage"
    printf '%s\n' "Installed AppImage to ${install_dir}/GSD-Dashboard.AppImage"
    ;;
  *)
    printf '%s\n' "Unsupported OS/arch: $os/$arch" >&2
    printf '%s\n' "Manual downloads: $manual_url" >&2
    exit 1
    ;;
esac
