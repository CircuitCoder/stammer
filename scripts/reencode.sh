#!/bin/bash
# converting all files in a dir to utf8 

for f in *.txt
do
  CHARSET="$( file -bi "$f"|awk -F "=" '{print $2}')"

  if [ "$CHARSET" == "unknown-8bit" ]; then
    CHARSET="GB18030"
  fi

  echo -e "\nConverting $f"
  iconv -f "$CHARSET" -t utf-8 "$f" -o "$f.converted"
done
