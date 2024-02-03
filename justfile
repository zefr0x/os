alias r := run

_default:
	@just --list

build:
	cargo build

run:
	cargo run

lint_all:
	pre-commit run --all-files

todo:
	rg "(.(TODO|FIXME|FIX|HACK|WARN|PREF|NOTE): )|(todo!)" --glob !{{ file_name(justfile()) }}
