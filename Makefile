run:
	cargo clippy && cargo build && cargo run

windows: ani-link.zip

ani-link.zip: ./target/x86_64-pc-windows-gnu/release/ani-link.exe mpv_source/mpv/build/libmpv-2.dll ./yt-dlp/yt-dlp.exe syncplay mpv/mpv.exe
	mkdir ani-link
	cp -r ./target/x86_64-pc-windows-gnu/release/ani-link.exe ./mpv_source/mpv/build/libmpv-2.dll ./yt-dlp/yt-dlp.exe ./mpv/mpv.exe ./syncplay/* ani-link
	zip -r ani-link.zip ani-link/*
	rm -rf ani-link

./target/x86_64-pc-windows-gnu/release/ani-link.exe: mpv_source/mpv/build/libmpv-2.dll
	MPV_SOURCE=mpv_source cargo build --release --target x86_64-pc-windows-gnu

mpv_source:
	mkdir -p mpv_source

mpv-dev.7z:
	bash -c 'curl -s https://api.github.com/repos/shinchiro/mpv-winbuild-cmake/releases/latest | grep "mpv-dev-x86_64-.*\.7z" | head -n1 | cut -d : -f 2,3 | tr -d \" | wget -o mpv-dev.7z -qi -; true'

mpv_source/64/libmpv-2.dll: mpv_source mpv-dev.7z
	7z x mpv-dev.7z -ompv_source/64

mpv_source/mpv/build/libmpv-2.dll: mpv_source/64/libmpv-2.dll
	mkdir -p mpv_source/mpv
	ln -sf ../64 mpv_source/mpv/build

yt-dlp:
	mkdir yt-dlp

yt-dlp.exe:
	bash -c 'curl -s https://api.github.com/repos/yt-dlp/yt-dlp/releases/latest | grep "/yt-dlp.exe" | head -n1 | cut -d : -f 2,3 | tr -d \" | wget -o yt-dlp.exe -qi -; true'

./yt-dlp/yt-dlp.exe: yt-dlp yt-dlp.exe
	cp yt-dlp.exe yt-dlp

syncplay.zip:
	bash -c 'curl -s https://api.github.com/repos/Syncplay/syncplay/releases/latest | grep "/.*Portable.zip" | head -n1 | cut -d : -f 2,3 | tr -d \" | wget -o syncplay.zip -qi -; true'

syncplay: syncplay.zip
	unzip syncplay.zip -d syncplay

mpv.7z:
	bash -c 'curl -s https://api.github.com/repos/shinchiro/mpv-winbuild-cmake/releases/latest | grep "/mpv-x86_64-.*\.7z" | head -n1 | cut -d : -f 2,3 | tr -d \" | wget -o mpv.7z -qi -; true'

mpv/mpv.exe: mpv.7z
	7z x mpv.7z -ompv

clean:
	rm -rf mpv_source yt-dlp mpv syncplay ani-link-* pkg *.zip *.7z *.exe
	cargo clean

.PHONY: clean windows run
