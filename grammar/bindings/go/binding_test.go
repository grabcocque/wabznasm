package tree_sitter_wabznasm_test

import (
	"testing"

	tree_sitter "github.com/tree-sitter/go-tree-sitter"
	tree_sitter_wabznasm "github.com/tree-sitter/tree-sitter-wabznasm/bindings/go"
)

func TestCanLoadGrammar(t *testing.T) {
	language := tree_sitter.NewLanguage(tree_sitter_wabznasm.Language())
	if language == nil {
		t.Errorf("Error loading Wabznasm grammar")
	}
}
