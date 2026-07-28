EXE ?= Dual

ifeq ($(OS),Windows_NT)
	NAME := $(EXE).exe
else
	NAME := $(EXE)
endif

build:
	cargo rustc --release -- -C target-cpu=native --emit link=$(NAME)
