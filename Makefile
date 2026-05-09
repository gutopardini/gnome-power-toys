.PHONY: install

PREFIX ?= $(HOME)/.local
BINDIR := $(PREFIX)/bin
APPDIR := $(PREFIX)/share/applications
METAINFODIR := $(PREFIX)/share/metainfo
ICONDIR_1024 := $(PREFIX)/share/icons/hicolor/1024x1024/apps
APP_ID := dev.gutopardini.GnomePowerToys

install:
	cargo build --release
	install -Dm755 target/release/gnome-power-toys $(BINDIR)/gnome-power-toys
	install -Dm644 data/$(APP_ID).desktop $(APPDIR)/$(APP_ID).desktop
	sed -i 's|^Exec=.*|Exec=$(BINDIR)/gnome-power-toys|' $(APPDIR)/$(APP_ID).desktop
	install -Dm644 data/$(APP_ID).metainfo.xml $(METAINFODIR)/$(APP_ID).metainfo.xml
	install -Dm644 assets/app-icons/gnome-power-toys-icon.svg $(PREFIX)/share/icons/hicolor/scalable/apps/$(APP_ID).svg
	rm -f $(ICONDIR_1024)/$(APP_ID).png
	-update-desktop-database $(APPDIR)
	@printf '%s\n' \
		'[Icon Theme]' \
		'Name=Hicolor' \
		'Comment=Fallback icon theme' \
		'Directories=scalable/apps,1024x1024/apps' \
		'' \
		'[scalable/apps]' \
		'Size=128' \
		'MinSize=16' \
		'MaxSize=1024' \
		'Type=Scalable' \
		'Context=Applications' \
		'' \
		'[1024x1024/apps]' \
		'Size=1024' \
		'Type=Fixed' \
		'Context=Applications' \
		> $(PREFIX)/share/icons/hicolor/index.theme
	-gtk-update-icon-cache $(PREFIX)/share/icons/hicolor
