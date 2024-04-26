/*
 * Copyright 2020 McGraw-Hill Education. All rights reserved. No reproduction or
 * distribution without the prior written consent of McGraw-Hill Education.
 */
#include <cstdint>
#include <nlohmann/json.hpp>
#include <sstream>

#include "common.h"
#include "framework2110.h"
using json = nlohmann::json;

int MAX_FAILURES = 8;

namespace framework2110 {
struct CLIArgs {
  bool json_output = false;
  uint32_t print_output = false;
  uint32_t asm_print_level = 0;
  uint32_t asm_print_level_override = false;
  uint32_t sim_print_level = 0;
  uint32_t sim_print_level_override = false;
  bool ignore_privilege = false;
  bool tester_verbose = false;
  uint64_t seed = 0;
  std::vector<std::string> test_filter;
};

std::function<void(Tester &)> setup = nullptr;
std::function<void(void)> shutdown = nullptr;
std::function<void(lc3::sim &)> testBringup = nullptr;
std::function<void(lc3::sim &)> testTeardown = nullptr;

int main(int argc, char *argv[]) {
  if (setup == nullptr || shutdown == nullptr || testBringup == nullptr ||
      testTeardown == nullptr) {
    // Will never happen if framework.h wrapper is used, since, if any of these
    // functions are missing, linking will fail.
    std::cerr << "Unit test does not implement necessary functionality.\n";
    return 0;
  }

  CLIArgs args;
  std::vector<std::pair<std::string, std::string>> parsed_args =
      parseCLIArgs(argc, argv);
  for (auto const &arg : parsed_args) {
    if (std::get<0>(arg) == "json-output") {
      args.json_output = true;
    } else if (std::get<0>(arg) == "print-output") {
      args.print_output = true;
    } else if (std::get<0>(arg) == "asm-print-level") {
      args.asm_print_level = std::stoi(std::get<1>(arg));
      args.asm_print_level_override = true;
    } else if (std::get<0>(arg) == "sim-print-level") {
      args.sim_print_level = std::stoi(std::get<1>(arg));
      args.sim_print_level_override = true;
      args.print_output = true;
    } else if (std::get<0>(arg) == "ignore-privilege") {
      args.ignore_privilege = true;
    } else if (std::get<0>(arg) == "tester-verbose") {
      args.tester_verbose = true;
    } else if (std::get<0>(arg) == "seed") {
      args.seed = std::stoull(std::get<1>(arg));
    } else if (std::get<0>(arg) == "test-filter") {
      args.test_filter.push_back(std::get<1>(arg));
    } else if (std::get<0>(arg) == "h" || std::get<0>(arg) == "help") {
      std::cout << "usage: " << argv[0] << " [OPTIONS] FILE [FILE...]\n";
      std::cout << "\n";
      std::cout << "  -h,--help              Print this message\n";
      std::cout
          << "  --json-output          Output test results in JSON format\n";
      std::cout << "  --print-output         Print program output\n";
      std::cout
          << "  --asm-print-level=N    Assembler output verbosity [0-9]\n";
      std::cout
          << "  --sim-print-level=N    Simulator output verbosity [0-9]\n";
      std::cout << "  --ignore-privilege     Ignore access violations\n";
      std::cout << "  --tester-verbose       Output debug messages\n";
      std::cout << "  --seed=N               Optional seed for randomization\n";
      std::cout << "  --test-filter=TEST     Only run TEST (can be repeated)\n";
      return 0;
    }
  }

  // suppress all assembler output if outputting json into stdout
  BufferedPrinter asm_printer = BufferedPrinter(!args.json_output);

  lc3::as assembler(asm_printer,
                    args.asm_print_level_override ? args.asm_print_level : 0,
                    false);
  lc3::conv converter(asm_printer,
                      args.asm_print_level_override ? args.asm_print_level : 0);
  lc3::core::SymbolTable symbol_table;

  std::vector<std::string> obj_filenames;
  bool valid_program = true;
  for (int i = 1; i < argc; i += 1) {
    std::string filename(argv[i]);
    if (filename[0] != '-') {
      lc3::optional<std::string> result;
      if (!endsWith(filename, ".obj")) {
        if (endsWith(filename, ".bin")) {
          result = converter.convertBin(filename);
        } else {
          lc3::optional<std::pair<std::string, lc3::core::SymbolTable>>
              asm_result;
          asm_result = assembler.assemble(filename);
          if (asm_result) {
            symbol_table.insert(asm_result->second.begin(),
                                asm_result->second.end());
            result = asm_result->first;
          }
        }
      } else {
        result = filename;
      }

      if (result) {
        obj_filenames.push_back(*result);
      } else {
        valid_program = false;
      }
    }
  }

  if (obj_filenames.size() == 0 || !valid_program) {
    if (args.json_output) {
      // we will insert the assembler output into the `error` key of the stdout
      // json if assembler fails at something
      std::ostringstream asm_printer_output;
      std::vector<char> buffer = asm_printer.getBuffer();
      for (auto it = buffer.begin(); it != buffer.end(); it++)
        asm_printer_output << *it;
      json out = {{"error", asm_printer_output.str()}};
      std::cout << out.dump(2, ' ', true) << std::endl;
      return 0;
    } else {
      return 1;
    }
  }

  if (valid_program) {
    Tester tester(args.print_output,
                  args.sim_print_level_override ? args.sim_print_level : 1,
                  args.ignore_privilege, args.tester_verbose, args.seed,
                  obj_filenames);
    tester.setSymbolTable(symbol_table);
    setup(tester);

    if (args.test_filter.size() == 0) {
      tester.testAll();
    } else {
      for (std::string const &test_name : args.test_filter) {
        tester.testSingle(test_name);
      }
    }

    if (args.json_output) {
      tester.printJson();
    } else {
      tester.printResults();
    }

    shutdown();
  }

  return 0;
}

TestCase::TestCase(std::string const &name, test_func_t test_func,
                   int randomizeSeed)
    : name(name), test_func(test_func), randomizeSeed(randomizeSeed) {}

Tester::Tester(bool print_output, uint32_t print_level, bool ignore_privilege,
               bool verbose, uint64_t seed,
               std::vector<std::string> const &obj_filenames)
    : print_output(print_output), ignore_privilege(ignore_privilege),
      verbose(verbose), print_level(print_level), seed(seed),
      obj_filenames(obj_filenames) {}

void Tester::registerTest(std::string const &name, test_func_t test_func,
                          int randomizeSeed) {
  tests.emplace_back(name, test_func, randomizeSeed);
}

void Tester::testAll(void) {
  for (TestCase const &test : tests) {
    curr_test_result = TestResult{};
    testSingle(test);
    test_results.push_back(curr_test_result);
  }
}

void Tester::testSingle(std::string const &test_name) {
  for (TestCase const &test : tests) {
    if (test.name == test_name) {
      return testSingle(test);
    }
  }
  return;
}

void Tester::testSingle(TestCase const &test) {
  // clear ostringstream for output of this test
  curr_output.str(std::string());

  BufferedPrinter printer(print_output);
  StringInputter inputter;
  lc3::sim simulator(printer, inputter, print_level);
  this->printer = &printer;
  this->inputter = &inputter;
  this->simulator = &simulator;

  curr_test_result.test_name = test.name;

  // seed cli arg >> test.randomizeSeed
  if (test.randomizeSeed >= 0) {
    if (seed == 0) {
      seed = simulator.randomizeState(test.randomizeSeed);
    } else {
      simulator.randomizeState(seed);
    }
    curr_test_result.seed = seed;
  } else {
    // if test.randomizeSeed is negative, don't randomize
    curr_test_result.seed = -1;
  }

  for (std::string const &obj_filename : obj_filenames) {
    auto res = simulator.loadObjFile(obj_filename);
    if (!res.first) {
      error("Simulator initialization failed", res.second);
      return;
    }
  }

  testBringup(simulator);

  if (ignore_privilege) {
    simulator.setIgnorePrivilege(true);
  }

  try {
    test.test_func(simulator, *this);
  } catch (Tester_error &te) {
    te.report(*this);
  } catch (std::exception &e) {
    error("Test case ran into exception", e.what());
    return;
  }

  curr_test_result.output = curr_output.str();

  testTeardown(simulator);

  this->printer = nullptr;
  this->inputter = nullptr;
  this->simulator = nullptr;
}

void Tester::appendTestPart(std::string const &label,
                            std::string const &message, bool pass) {
  TestPart *test_part = new TestPart{};
  test_part->label = label;
  test_part->message = message;
  if (!pass) {
    curr_test_result.fail_inds.insert(curr_test_result.parts.size());
  }
  curr_test_result.parts.push_back(test_part);
}

void Tester::output(std::string const &message) {
  curr_output << message << "\n";
}

void Tester::debugOutput(std::string const &message) {
  if (verbose) {
    std::cout << " " << message << "\n";
  }
}

void Tester::error(std::string const &label, std::string const &message) {
  TestPart *error = new TestPart{};
  error->label = label;
  error->message = message;
  curr_test_result.error = error;
}

void Tester::printResults() {
  for (const TestResult &test_result : test_results) {
    std::cout << "==========\n";
    std::cout << "Test: " << test_result.test_name;
    if (test_result.seed != -1) {
      std::cout << " (Randomized Machine, Seed: " << seed << ")";
    }
    std::cout << std::endl;
    std::cout << test_result.output;
    auto error = test_result.error;
    if (error) {
      std::cout << "ERROR: " << error->label << ":\n"
                << error->message << std::endl;
      continue;
    }
    auto parts = test_result.parts;
    for (int i = 0; i < parts.size(); i++) {
      auto part = parts.at(i);
      bool part_passed = test_result.fail_inds.count(i) == 0;
      std::cout << (part_passed ? "--" : "!!") << part->label << " => ";
      std::cout << (part_passed ? "Pass" : "FAIL") << std::endl;
      if (!part->message.empty())
        std::cout << part->message << std::endl;
    }
  }
}

void Tester::printJson() {
  // json output mimics `--zucchini` flag output from circuitsim-tester
  json test_results_json = json::array();
  for (const TestResult &test_result : test_results) {
    json test_result_json;
    json partial_fails = json::array();
    test_result_json["testName"] = test_result.test_name;
    test_result_json["seed"] = test_result.seed;
    test_result_json["output"] = test_result.output;

    // if there is an error, report it as the single failure and test
    // so that no points will be awarded
    if (test_result.error) {
      json e = {
          {"displayName", test_result.error->label},
          {"message", test_result.error->message},
      };
      partial_fails.push_back(e);
      test_result_json["partialFailures"] = partial_fails;
      test_result_json["total"] = 1;
      test_result_json["failed"] = test_result_json["total"];
      test_results_json.push_back(test_result_json);
      continue;
    }

    test_result_json["total"] = test_result.parts.size();
    int num_fails = test_result.fail_inds.size();
    test_result_json["failed"] = num_fails;
    if (num_fails > 0) {
      auto parts = test_result.parts;
      int i = 0;
      int c = 0;
      while (i < parts.size() && c < MAX_FAILURES) {
        if (test_result.fail_inds.count(i) == 0) {
          i++;
          continue;
        }
        auto part = parts.at(i);
        json part_json = {
            {"displayName", part->label},
            {"message", part->message},
        };
        partial_fails.push_back(part_json);
        i++;
        c++;
      }
    }
    test_result_json["partialFailures"] = partial_fails;
    test_results_json.push_back(test_result_json);
  }
  json out;
  out["tests"] = test_results_json;
  std::cout << out.dump(2, ' ', true) << std::endl;
}

std::string Tester::getConsoleOutput(void) const {
  auto const &buffer = printer->getBuffer();
  return std::string{buffer.begin(), buffer.end()};
}

uint16_t Tester::get_symbol_location(const std::string &symbol) {
  const auto symbol_addr = symbol_table.find(symbol);
  if (symbol_addr == symbol_table.end())
    throw Tester_error{"Checking address of " + symbol + " in symbol table",
                       "There is no " + symbol + " label in the code."};
  return symbol_addr->second;
}

void Tester::write_mem_at_symbol(const std::string &symbol, std::uint16_t val) {
  const auto symbol_addr = get_symbol_location(symbol);
  simulator->writeMem(symbol_addr, val);
}

std::uint16_t Tester::read_mem_at_symbol(const std::string &symbol) {
  const auto symbol_addr = get_symbol_location(symbol);
  return simulator->readMem(symbol_addr);
}

void Tester::write_string_at_symbol(const std::string &symbol,
                                    const std::string &str) {
  const auto symbol_addr = get_symbol_location(symbol);
  return simulator->writeStringMem(symbol_addr, str);
}

std::string Tester::read_mem_string(std::uint16_t addr) {
  std::string str{};
  const int max_chars = 100;
  for (; str.size() < max_chars && simulator->readMem(addr); ++addr) {
    auto c = simulator->readMem(addr);
    if (c > 127)
      throw Tester_error("Invalid string read",
                         (std::ostringstream{} << "Character at address "
                                               << std::showbase << std::hex
                                               << addr << " is invalid (>127)")
                             .str());
    str.push_back(c);
  }
  return str;
}

// Overload for specified length
std::string Tester::read_mem_string(std::uint16_t addr, std::size_t len) {
  std::string str{};
  for (std::size_t i = 0; i != len; ++i) {
    auto c = simulator->readMem(addr++);
    if (c > 127)
      throw Tester_error("Invalid string read",
                         (std::ostringstream{}
                          << "Character at address " << std::showbase
                          << std::hex << (addr - 1) << " is invalid (>127)")
                             .str());
    str.push_back(c);
  }
  return str;
}

}; // namespace framework2110
