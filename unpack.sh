#!/bin/bash
dir=$(pwd)/bin;
if ! [ -d "$dir" ]; then
    mkdir $dir  
fi
pushd $dir
:${arch:=$(uname -m)}
if [ "$arch" = "aarch64" ]; then
 arch=armv8; 
else
 arch=$(uname -m);
fi

wget https://github.com/mazige1997/dockeropenwrt/blob/main/nasxunlei-DSM7-${arch}.spk

tar --wildcards -Oxf $(find . -type f -name \*-${arch}.spk | head -n1) package.tgz | tar --wildcards -xJC ${dir} 'bin/bin/*' 'ui/index.cgi'
mv ${dir}/bin/bin/* ${dir}/
mv ${dir}/ui/index.cgi ${dir}/xunlei-pan-cli-web
rm -rf ${dir}/bin/bin
rm -rf ${dir}/bin
rm -rf ${dir}/ui
rm -f ${dir}/version_code ${dir}/*.spk
popd
