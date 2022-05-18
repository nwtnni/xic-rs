#!/bin/sh
#
# This is a very simple script that uses gcc to link in a given .s
# file to the xi runtime library, and uses xifilt to help 
# decode error messages
#
# Use this like ./linkxi.sh -o binary foo.s
#
DIR=$(dirname $0)
RTDIR=$DIR/../runtime
ABI_FLAG=$($RTDIR/platform-flags.sh)

# echo "ABI_FLAG = $ABI_FLAG"

gcc $ABI_FLAG "$@" -L$DIR -L$RTDIR \
		-lQtXi -lxi -lQtGui -lQtCore -lpthread -lstdc++ 2>&1 \
	| $RTDIR/xifilt
