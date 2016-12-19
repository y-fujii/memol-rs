.PHONY: all
all:
	rm -rf tutorial
	lilypond-book --format=html --output=tutorial tutorial.ly
	mv tutorial/tutorial.html tutorial/index.html

.PHONY: spell
spell:
	aspell --mode=html list < tutorial.ly | sort | uniq
