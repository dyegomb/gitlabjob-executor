#!/usr/bin/env python3

import unittest
import utils
from os import environ

class Utils(unittest.TestCase):
    def test_loadconf(self):
        try:
            utils.load_conf("tests/test.env")
            self.assertTrue(True)
        except Exception as error:
            self.assertTrue(False, 
                "Error while loading config file: "+str(error))

    def test_nofile(self):
        result = utils.load_conf("nofile")
        self.assertFalse(result)


    def test_notoverwrite(self):
        environ["Teste1"] = "setted"
        utils.load_conf("tests/test.env")

        self.assertEqual(environ.get("Teste1"), "setted")

    def test_quotationmarks(self):
        utils.load_conf("tests/test.env")

        self.assertEqual(environ.get("Teste3"), '="1$23')