#! /usr/bin/env bash
xxd -e -c 4 $1 | awk '{print "0x" $2 ","}'
