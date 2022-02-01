#!/usr/bin/env python3
from configparser import ConfigParser
from itertools import chain
import os

def load_conf(file=".env") -> bool :
    if not os.path.exists(file):
        return False
    config = ConfigParser()
    config.optionxform = str # Enable case sensitive keys
    with open(file, 'r') as rfile:
        config.read_file(chain(['[DummyHeader]'], rfile))

    for item in config['DummyHeader'].items():
        key = item[0]
        if not os.environ.get(key):
            continue

        value = item[1]

        if value.startswith('"') and value.endswith('"'):
            value = value.lstrip('"').rstrip('"')

        os.environ[key] = value

    return True
