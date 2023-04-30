#!/bin/bash
id=$(cat /proc/sys/kernel/random/uuid | cut -c1-7)
echo "unique=\"synology_${id}_720+\"" > ./files/synoinfo.conf 