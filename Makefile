.PHONY: setup dev preview check build doctor
setup:
	./scripts/setup.sh
dev:
	./scripts/dev.sh
preview:
	./scripts/dev.sh --preview home
check:
	./scripts/check.sh
build:
	./scripts/build.sh
doctor:
	./scripts/doctor.sh
