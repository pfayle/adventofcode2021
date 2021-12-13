#!/bin/bash
set -e
nasm -f macho64 -g main.asm
gcc -g -m64 -nostartfiles -o main main.o

./main < input.txt

# Tests
#echo 0000 | ./main # expect 0000, 1111, 0
#echo 0100 | ./main # expect 0100, 1011, 44
#echo -e "0000\n0001\n0011\n0111\n" | ./main
#echo 1111 | ./main
#echo 000 | ./main
#echo 010 | ./main
#echo 100 | ./main
#echo 110 | ./main
#echo 111 | ./main
#echo -e "001\n001\n111" | ./main
#echo -e "0011\n0011" | ./main
#echo -e "0011\n0011\n1111" | ./main
#echo 0001 | ./main # expect 14
#echo 0000001 | ./main
#echo 00000001 | ./main
#echo 000000001 | ./main
#echo 0000000001 | ./main
#echo 00000000001 | ./main
#echo 000000000001 | ./main # expect 4094
#echo 0000000000001 | ./main # undefined
#echo -e "010000100111\n010000100111" | ./main # expect 3223016
