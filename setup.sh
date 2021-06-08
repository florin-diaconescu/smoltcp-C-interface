#!/bin/bash

usage_and_exit() {
	echo "Usage: $0 <kernel-path>"
	exit -2
}

# Check params
[ "$#" -eq 0 ] && usage_and_exit

KERNEL=$1

brctl addbr br0
ip addr flush dev eno1
brctl addif br0 eno1
tunctl -t tap0 -u `whoami`
brctl addif br0 tap0
ifconfig eno1 up
ifconfig tap0 up
ifconfig br0 up
dhclient -v br0

qemu-system-x86_64 -cpu host -enable-kvm -m 256M -nodefaults -no-acpi -display none -serial stdio -device isa-debug-exit -netdev tap,id=mynet0,ifname=tap0,script=no,downscript=no -device virtio-net-pci,netdev=mynet0,mac=52:55:00:d1:55:01 -kernel $KERNEL
