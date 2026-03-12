# Mantainer: Adrián Nicolás <nicolas.aguilera.adrian@gmail.com>

pkgname=ani-link
pkgver=0.4.1
pkgrel=2
pkgdesc="Anime scraper"
arch=('x86_64')
url="https://github.com/adriannic/ani-link"
makedepends=('rust')
depends=('mpv' 'yt-dlp')
optdepends=('syncplay: for syncplay support')
source=()
options=(!debug)

build() {
  cd "$startdir"
  cargo build --release
}

package () {
  cd "$startdir"
  install -Dm755 target/release/ani-link "$pkgdir"/usr/bin/ani-link
}
