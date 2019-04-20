#!/bin/bash
# converting all files in a dir to utf8 

for f in *
do
  echo -e "\nConverting $f"
  CHARSET="$( file -bi "$f"|awk -F "=" '{print $2}')"
  if [ "$CHARSET" != utf-8 ]; then
    iconv -f "$CHARSET" -t utf-8 "$f" -o "$f"
  fi
done
