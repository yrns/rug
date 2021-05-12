.PHONY: all test clean clean-steps

all: mal/Makefile mal/impls/rug test

mal/Makefile:
	git clone --depth=1 git@github.com:kanaka/mal.git
	(cd $(dir $@); git apply ../mal_rug.patch)

mal/impls/rug: mal/Makefile
	mkdir -p $@
	ln -s -r -t $@ $(wildcard impl/*)

test:
	(cd mal; make "test^rug")

clean:
	rm -rf mal

# cd mal; make "clean^rug" also will clean cargo...
clean-steps:
	rm -f $(wildcard mal/impls/rug/step*)
