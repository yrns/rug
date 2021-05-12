.PHONY: all test clean clean-steps

all: mal/Makefile mal/impls/rug test

mal/Makefile:
	git clone --depth=1 git@github.com:kanaka/mal.git
	(cd $(dir $@); git apply ../mal_rug.patch)

mal/impls/rug: mal/Makefile
	mkdir -p $@
	ln -s -r -t $@ $(wildcard impl/*)

test:
	(cd mal; make "test^rug^step0")

clean:
	rm -rf mal

clean-steps:
	rm $(wildcard mal/impls/rug/step*)
