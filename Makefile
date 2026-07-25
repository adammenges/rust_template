.DEFAULT_GOAL := help

.PHONY: help setup setup-linux doctor dev check icons build-app build-macos build-linux clean

help:
	@echo "Rust + Tauri desktop template"
	@echo ""
	@echo "  make setup      Install pinned tools and fetch dependencies"
	@echo "  make setup-linux Install Linux system libraries (Linux only)"
	@echo "  make doctor     Check the local development environment"
	@echo "  make dev        Launch the app with hot reload"
	@echo "  make check      Run syntax checks, formatting, Clippy, and tests"
	@echo "  make icons      Regenerate platform icons from the source PNG"
	@echo "  make build-app  Build verified packages for the current host"
	@echo "  make build-macos Build an ARM64 .app (macOS only)"
	@echo "  make build-linux Build .deb and AppImage packages (Linux only)"
	@echo "  make clean      Remove generated build output"

setup:
	./scripts/setup.sh

setup-linux:
	./scripts/install_linux_dependencies.sh

doctor:
	./scripts/doctor.sh

dev:
	./scripts/dev.sh

check:
	./scripts/check.sh

icons:
	cargo tauri icon assets/icons/AppIcon-1024.png

build-app:
	./scripts/build.sh

build-macos:
	./scripts/build_macos_app.sh

build-linux:
	./scripts/build_linux_app.sh

clean:
	cargo clean
	rm -rf dist
