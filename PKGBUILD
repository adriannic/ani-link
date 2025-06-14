# Mantainer: Adrián Nicolás <nicolas.aguilera.adrian@gmail.com>

pkgname=ani-link
pkgver=0.3.2
pkgrel=1
pkgdesc="Anime scraper"
arch=('x86_64')
url="https://github.com/adriannic/ani-link"
makedepends=('rust')
optdepends=('mpv: to open episodes in mpv' 'yt-dlp: to open episodes in mpv')
source=()
options=(!debug)

build() {
  cd "$startdir"
  cargo build --release --locked
}

package () {
  cd "$startdir"
  install -Dm755 target/release/ani-link "$pkgdir"/usr/bin/ani-link
}
