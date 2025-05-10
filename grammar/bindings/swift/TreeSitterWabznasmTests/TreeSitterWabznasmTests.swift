import XCTest
import SwiftTreeSitter
import TreeSitterWabznasm

final class TreeSitterWabznasmTests: XCTestCase {
    func testCanLoadGrammar() throws {
        let parser = Parser()
        let language = Language(language: tree_sitter_wabznasm())
        XCTAssertNoThrow(try parser.setLanguage(language),
                         "Error loading Wabznasm grammar")
    }
}
