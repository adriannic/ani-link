
ani-link.zip: ./target/x86_64-pc-windows-gnu/release/ani-link.exe mpv_source/64/libmpv-2.dll yt-dlp/yt-dlp.exe syncplay mpv/mpv.exe
	mkdir ani-link
	cp -r ./target/x86_64-pc-windows-gnu/release/ani-link.exe ./mpv_source/64/libmpv-2.dll ./yt-dlp/yt-dlp.exe ./mpv/mpv.exe ./syncplay/* ani-link
	zip -r ani-link.zip ani-link/*
	rm -rf ani-link

./target/x86_64-pc-windows-gnu/release/ani-link.exe: mpv_source/64/libmpv-2.dll
	MPV_SOURCE=mpv_source cargo build --release --target x86_64-pc-windows-gnu

mpv_source/64/libmpv-2.dll:
	mkdir mpv_source
	bash -c 'curl -s https://api.github.com/repos/shinchiro/mpv-winbuild-cmake/releases/latest | grep "mpv-dev-x86_64-[!v].*\.7z" | cut -d : -f 2,3 | tr -d \" | wget -qi -; true'
	ls . | grep 'mpv-.*' | xargs -I = -- 7z x = -ompv_source/64
	ls . | grep 'mpv-.*' | xargs -I = -- rm -rf =
	mkdir mpv_source/mpv
	ln -s ../64 mpv_source/mpv/build

yt-dlp/yt-dlp.exe:
	mkdir yt-dlp
	bash -c 'curl -s https://api.github.com/repos/yt-dlp/yt-dlp/releases/latest | grep "yt-dlp.exe" | cut -d : -f 2,3 | tr -d \" | wget -qi -; true'
	mv yt-dlp.exe yt-dlp

syncplay:
	bash -c 'curl -s https://api.github.com/repos/Syncplay/syncplay/releases/latest | grep ".*Portable.zip" | cut -d : -f 2,3 | tr -d \" | wget -qi -; true'
	ls . | grep '.*Portable.zip' | xargs -I = -- unzip = -d syncplay
	ls . | grep '.*Portable.zip' | xargs -I = -- rm -rf =

mpv/mpv.exe:
	bash -c 'curl -s https://api.github.com/repos/shinchiro/mpv-winbuild-cmake/releases/latest | grep "mpv-x86_64-[!v].*\.7z" | cut -d : -f 2,3 | tr -d \" | wget -qi -; true'
	ls . | grep 'mpv-.*' | xargs -I = -- 7z x = -ompv
	ls . | grep 'mpv-.*' | xargs -I = -- rm -rf =

clean:
	rm -rf mpv_source yt-dlp mpv syncplay ani-link.zip ani-link-* pkg
	cargo clean

.PHONY: clean
