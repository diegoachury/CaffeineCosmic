# Instalación de cosmic-caffeine. Usado tanto por el build de Flatpak
# (PREFIX=/app) como por la instalación local (PREFIX=$HOME/.local).

PREFIX  ?= /usr
DESTDIR ?=

APPID = io.github.diegoachury.CaffeineCosmic
BIN   = cosmic-caffeine

ICONDIR   = $(DESTDIR)$(PREFIX)/share/icons/hicolor/scalable
APPSDIR   = $(DESTDIR)$(PREFIX)/share/applications
METADIR   = $(DESTDIR)$(PREFIX)/share/metainfo
BINDIR    = $(DESTDIR)$(PREFIX)/bin

.PHONY: all build install uninstall clean

all: build

build:
	cargo build --release

install:
	install -Dm0755 target/release/$(BIN) $(BINDIR)/$(BIN)
	install -Dm0644 data/$(APPID).desktop $(APPSDIR)/$(APPID).desktop
	install -Dm0644 data/$(APPID).metainfo.xml $(METADIR)/$(APPID).metainfo.xml
	install -Dm0644 data/icons/hicolor/scalable/apps/$(APPID).svg $(ICONDIR)/apps/$(APPID).svg
	install -Dm0644 data/icons/hicolor/scalable/status/$(APPID)-active-symbolic.svg $(ICONDIR)/status/$(APPID)-active-symbolic.svg
	install -Dm0644 data/icons/hicolor/scalable/status/$(APPID)-inactive-symbolic.svg $(ICONDIR)/status/$(APPID)-inactive-symbolic.svg

uninstall:
	rm -f $(BINDIR)/$(BIN)
	rm -f $(APPSDIR)/$(APPID).desktop
	rm -f $(METADIR)/$(APPID).metainfo.xml
	rm -f $(ICONDIR)/apps/$(APPID).svg
	rm -f $(ICONDIR)/status/$(APPID)-active-symbolic.svg
	rm -f $(ICONDIR)/status/$(APPID)-inactive-symbolic.svg

clean:
	cargo clean
