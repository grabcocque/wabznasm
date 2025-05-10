from unittest import TestCase

import tree_sitter
import tree_sitter_wabznasm


class TestLanguage(TestCase):
    def test_can_load_grammar(self):
        try:
            tree_sitter.Language(tree_sitter_wabznasm.language())
        except Exception:
            self.fail("Error loading Wabznasm grammar")
