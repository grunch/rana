#!/bin/bash
file="archs"
manifest="manifest.txt"
arch=`cat $file`
for i in $arch; do
    echo "Cross compiling for $i"
    cross build --release --target $i
    filename=`basename $PWD`
    if [ $i == "x86_64-pc-windows-gnu" ]; then
        filename=`basename $PWD`".exe"
    fi
    cd target/$i/release
    mkdir $i
    cp $filename $i/
    sha256sum $i/$filename >> ../../../bin/$manifest
    tar -cjf rana-$i.tar.gz $i
    sha256sum rana-$i.tar.gz >> ../../../bin/$manifest
    mv rana-$i.tar.gz ../../../bin
    rm -rf $i
    cd ../../../
done