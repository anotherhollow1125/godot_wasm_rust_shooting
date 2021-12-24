#!/usr/bin/bash

# バカよけ
if [ "$(basename `pwd`)" != "docs" ]; then
    echo "$(basename `pwd`)"
    echo "ther is not docs directory"
    exit 1
fi

ls | grep -v "CNAME" | xargs rm -rf
cp ../shooting_gdt/web/* .