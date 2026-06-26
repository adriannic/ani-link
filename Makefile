
./target/x86_64-pc-windows-gnu/release/ani-link.exe: mpv_source
	MPV_SOURCE=mpv_source cargo build --release --target x86_64-pc-windows-gnu

mpv_source:
	mkdir mpv_source
	bash -c 'curl -s https://api.github.com/repos/shinchiro/mpv-winbuild-cmake/releases/latest | grep "mpv-dev-x86_64-[!v].*\.7z" | cut -d : -f 2,3 | tr -d \" | wget -qi -; true'
	ls . | grep 'mpv-.*' | xargs -I = -- 7z x = -ompv_source/64
	ls . | grep 'mpv-.*' | xargs -I = -- rm -rf =
	mkdir mpv_source/mpv
	ln -s ../64 mpv_source/mpv/build

clean:
	rm -rf mpv_source
	cargo clean

.PHONY: clean windows
