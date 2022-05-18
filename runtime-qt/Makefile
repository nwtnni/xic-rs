CFLAGS=-g -Wall -O2 -m64 -fno-stack-protector
#CFLAGS=-g -Wall -O2 -fno-stack-protector
CC=gcc

# QtXi 
QTXIDIR = libQtXi
QTXILIB = libQtXi.a

ABI_FLAG=$(shell ./platform-flags.sh)

all: $(QTXILIB)

$(QTXILIB): libxi.deps
	cd $(QTXIDIR); cmake .; make; 
	cp $(QTXIDIR)/qt.ixi include/
	cp $(QTXIDIR)/$(QTXILIB) .

libxi.deps: ../runtime/libxi/*.h
	cp ../runtime/libxi/*.h ./libQtXi/
	@touch libxi.deps

clean:
	rm -rf $(QTXIDIR)/{CMakeCache.txt,CMakefiles}
	rm -f $(QTXILIB) $(QTXIIF)

clobber: clean

dust: clobber

FILES = runtime/linkqt.sh $(QTXILIBDIR)

tarballs: $(GCLIBDIR)/configure
	make clean
	cd .. && tar cvz --exclude=CVS --exclude="*~" ${FILES} > runtime.tar.gz
	cd .. && zip runtime.zip -r ${FILES} -x '*CVS*' -x "*~"

.PHONY: libxi
