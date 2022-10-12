
import sys
import platform
import unittest

PYTHON_VERSION = "3.10.7"


class Py3Test(unittest.TestCase):

    def test_version(self):
        self.assertEqual(platform.python_version(), PYTHON_VERSION)

    def test_interpreter(self):
        self.assertIn('python3_test.runfiles/python3_interpreter/python/install/bin/', sys.executable)

if __name__ == "__main__":
    unittest.main()
