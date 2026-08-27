EXE ?= Dual

ifeq ($(OS),Windows_NT)
	NAME := $(EXE).exe
else
	NAME := $(EXE)
endif

build:
	cargo rustc --release --features tuning -- -C target-cpu=native --emit link=$(NAME)