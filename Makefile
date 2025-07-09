DEST=~/.config/qutebrowser/userscripts/fetch-creds

all: build

build:
	cargo build -r

clean: build
	rm -rf target

install: build
	cp target/release/qute_userscript $(DEST)
