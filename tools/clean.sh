#!/bin/bash

set -e

sed -e '/Game BoyTM CPU Manual/d' -e '/Page.*/d' -e '/Instruction Param.*/d' -e '/^[[:space:]]*$/d' gb_opcode_rip.txt
