/*
 * Copyright 2020 McGraw-Hill Education. All rights reserved. No reproduction or
 * distribution without the prior written consent of McGraw-Hill Education.
 */
#include <cstdint>
#include <functional>
#include <sstream>
#include <string>
#include <unordered_set>
#include <vector>

#include "framework_common.h"

namespace framework2110 {
class Tester;

using test_func_t = std::function<void(lc3::sim &, Tester &)>;

extern std::function<void(Tester &)> setup;
extern std::function<void(void)> shutdown;
extern std::function<void(lc3::sim &)> testBringup;
extern std::function<void(lc3::sim &)> testTeardown;

struct TestCase {
  std::string name;
  test_func_t test_func;
  // double points;
  bool randomize;

  TestCase(std::string const &name, test_func_t test_func, bool randomize);

  // TestCase(std::string const &name, test_func_t test_func, double points,
  //          bool randomize);
};

struct TestPart {
  std::string label;
  std::string message;
};

struct TestResult {
  std::string test_name;
  std::vector<TestPart *> parts;
  std::unordered_set<int> fail_inds;
  std::string output;
  TestPart *error;
  double seed;
};

class Tester {
private:
  std::vector<TestCase> tests;
  bool print_output, ignore_privilege, verbose;
  uint32_t print_level;
  uint64_t seed;
  std::vector<std::string> obj_filenames;
  lc3::core::SymbolTable symbol_table;

  BufferedPrinter *printer;
  StringInputter *inputter;
  lc3::sim *simulator;

  std::vector<TestResult> test_results;
  TestResult curr_test_result;
  std::ostringstream curr_output;

  void testAll(void);
  void testSingle(std::string const &test_name);

  void testSingle(TestCase const &test);

  void appendTestPart(std::string const &label, std::string const &message,
                      bool pass);

  void printResults();

  void printJson();

  double checkSimilarityHelper(std::vector<char> const &source,
                               std::vector<char> const &target) const;

  friend int main(int argc, char *argv[]);

public:
  enum PreprocessType {
    IgnoreCase = 1,
    IgnoreWhitespace = 2,
    IgnorePunctuation = 4
  };

  Tester(bool print_output, uint32_t print_level, bool ignore_privilege,
         bool verbose, uint64_t seed,
         std::vector<std::string> const &obj_filenames);

  // // overload for compatibility with v2 tester API
  // void registerTest(std::string const &name, test_func_t test_func,
  //                   double points, bool randomize) {
  //   registerTest(name, test_func, randomize);
  // };

  void registerTest(std::string const &name, test_func_t test_func,
                    bool randomize);

  template <typename T>
  void verify(std::string const &label, T out, T expected, bool (*comp)(T, T),
              std::string (*print)(T));
  template <typename T>
  void verify(std::string const &label, T out, T expected);

  // // overload for compatibility with v2 tester API
  // void verify(std::string const &label, bool pass, double points) {
  //   verify(label, pass);
  // };

  void verify(std::string const &label, bool pass);

  void output(std::string const &message);
  void debugOutput(std::string const &message);
  void error(std::string const &label, std::string const &message);

  void setInputString(std::string const &source) {
    inputter->setString(source);
  }
  void setInputCharDelay(uint32_t inst_count) {
    inputter->setCharDelay(inst_count);
  }

  std::string getOutput(void) const;
  void clearOutput(void) { printer->clear(); }
  bool checkMatch(std::string const &a, std::string const &b) const {
    return a == b;
  }
  bool checkContain(std::string const &str,
                    std::string const &expected_part) const;
  double checkSimilarity(std::string const &source,
                         std::string const &target) const;
  std::string getPreprocessedString(std::string const &str,
                                    uint64_t type) const;

  lc3::core::SymbolTable const &getSymbolTable(void) const {
    return symbol_table;
  }

private:
  void setSymbolTable(lc3::core::SymbolTable const &symbol_table) {
    this->symbol_table = symbol_table;
  }
  // friend int framework2110::main(int argc, char *argv[]);
};

class Tester_error final {
  std::string lbl, msg;

public:
  Tester_error(std::string l, std::string m) : lbl{l}, msg{m} {}
  void report(Tester &tester) const noexcept { tester.error(lbl, msg); }
};

int main(int argc, char *argv[]);
}; // namespace framework2110
