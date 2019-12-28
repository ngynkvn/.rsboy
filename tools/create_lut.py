#!/usr/bin/python3
import re
import itertools
import json


def get_indices(txt, sections):
    indices = {}
    for section, regex in sections:
        indices[section] = [i for i, v in enumerate(txt) if re.match(regex, v)]
    return indices


info = {}
lut = {}
full_lut = {}

with open("gb_opcodes.txt") as f:
    txt = f.read().strip().split("\n")
    regexes = {
        "Section": r"(\d{1,}\.){3} ",
        "Subsection": r"\d{1,}\. ",
        "Description": r"Description:",
        "Use with": r"Use with:",
        "Opcodes": r"Opcodes:",
        "Flags affected": r"Flags affected:",
    }
    # Get the indexes that match a certain regex.
    # This will be used to create a tree structure
    # that will be used to store information about the
    # different types of opcodes on the gameboy.
    indices = get_indices(txt, regexes.items())
    markers = sorted(i for k, v in indices.items() for i in zip(v, itertools.repeat(k)))
    markers.append((len(txt), None))
    section_title = None
    subtitle = None
    i = 0
    while i < len(markers) - 1:
        start, title = markers[i]
        end, _ = markers[i + 1]
        if title == "Section":
            section_title = re.sub(regexes[title], "", txt[start])
            info[section_title] = {}
        elif title == "Subsection":
            subtitle = re.sub(regexes[title], "", txt[start])
            info[section_title][subtitle] = {}
            ptr = info[section_title][subtitle]
        elif title == "Description":
            ptr[title] = re.sub(regexes[title], "", " ".join(txt[start:end])).strip()
        elif title == "Use with":
            ptr[title] = re.sub(regexes[title], "", " ".join(txt[start:end])).strip()
        elif title == "Opcodes":
            ptr[title] = re.sub(regexes[title], "", "\n".join(txt[start:end])).strip()
            opcodes = []
            for op in txt[start + 1 : end]:
                op = op.split(" ")
                if len(op) == 5:
                    opcodes.append(f"{op[2]} {op[3]}")
                elif len(op) == 4:
                    opcodes.append(op[2])
            lut.update(zip(opcodes, itertools.repeat([section_title, subtitle])))
            full_lut.update(zip(opcodes, itertools.repeat(ptr)))
        elif title == "Flags affected":
            ptr[title] = re.sub(regexes[title], "", " ".join(txt[start:end])).strip()
        i += 1
    # from pprint import pprint
    # pprint(info)
    # pprint(full_lut)

json.dump(full_lut, open("lookup.json", "w"))
print("Done!")
