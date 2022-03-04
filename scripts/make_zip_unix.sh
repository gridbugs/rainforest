#!/usr/bin/env bash
set -euxo pipefail

echo $MODE
echo $ZIP_NAME

TMP=$(mktemp -d)
mkdir $TMP/$ZIP_NAME
cp -v target/$MODE/rainforest_wgpu $TMP/$ZIP_NAME/rainforest-graphical
cp -v target/$MODE/rainforest_ggez $TMP/$ZIP_NAME/rainforest-graphical-compatibility
cp -v target/$MODE/rainforest_ansi_terminal $TMP/$ZIP_NAME/rainforest-terminal

cp -v extras/unix/* $TMP/$ZIP_NAME

pushd $TMP
zip $ZIP_NAME.zip $ZIP_NAME/*
popd
mv $TMP/$ZIP_NAME.zip .
