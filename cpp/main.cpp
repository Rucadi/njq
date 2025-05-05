#include <iostream>
#include <sstream>
#include <string>
#include <vector>
#include <stdexcept>
#include "libsnix.h"

void print_usage(const char *prog) {
    std::cerr << "Usage: " << prog << " [--raw] [--nix] <nix_expr> [json_file]\n";
    std::cerr << "  --raw        Print output without JSON escapes\n";
    std::cerr << "  --nix        Treat <nix_expr> as a self-contained expression (skip JSON input)\n";
    std::cerr << "  <nix_expr>   The Nix expression to evaluate (quoted)\n";
    std::cerr << "  [json_file]  Path to JSON input file; if omitted, reads from stdin\n";
    std::cerr << "  help         Show this help message\n";
    std::exit(EXIT_FAILURE);
}

std::string slurp_stdin() {
    std::ostringstream ss;
    ss << std::cin.rdbuf();
    return ss.str();
}

std::string nix_string_literal(const std::string &s) {
    std::string out = "''";
    for (char c : s) {
        if (c == '\'') out += "\\'";
        else out += c;
    }
    out += "''";
    return out;
}

void append_utf8(std::string &out, unsigned int codepoint) {
    if (codepoint <= 0x7F) out += static_cast<char>(codepoint);
    else if (codepoint <= 0x7FF) {
        out += static_cast<char>(0xC0 | ((codepoint >> 6) & 0x1F));
        out += static_cast<char>(0x80 | (codepoint & 0x3F));
    } else if (codepoint <= 0xFFFF) {
        out += static_cast<char>(0xE0 | ((codepoint >> 12) & 0x0F));
        out += static_cast<char>(0x80 | ((codepoint >> 6) & 0x3F));
        out += static_cast<char>(0x80 | (codepoint & 0x3F));
    } else if (codepoint <= 0x10FFFF) {
        out += static_cast<char>(0xF0 | ((codepoint >> 18) & 0x07));
        out += static_cast<char>(0x80 | ((codepoint >> 12) & 0x3F));
        out += static_cast<char>(0x80 | ((codepoint >> 6) & 0x3F));
        out += static_cast<char>(0x80 | (codepoint & 0x3F));
    }
}

std::string unescape_json(const std::string &s) {
    std::string out;
    out.reserve(s.size());
    for (size_t i = 0; i < s.size(); ++i) {
        char c = s[i];
        if (c == '\\' && i + 1 < s.size()) {
            char next = s[++i];
            switch (next) {
                case '"': out += '"'; break;
                case '\\': out += '\\'; break;
                case '/':  out += '/';  break;
                case 'b':  out += '\b'; break;
                case 'f':  out += '\f'; break;
                case 'n':  out += '\n'; break;
                case 'r':  out += '\r'; break;
                case 't':  out += '\t'; break;
                case 'u': {
                    if (i + 4 < s.size()) {
                        unsigned int code = 0;
                        for (int k = 0; k < 4; ++k) {
                            char h = s[++i];
                            code <<= 4;
                            if (h >= '0' && h <= '9') code |= (h - '0');
                            else if (h >= 'A' && h <= 'F') code |= (10 + h - 'A');
                            else if (h >= 'a' && h <= 'f') code |= (10 + h - 'a');
                            else throw std::runtime_error("Invalid unicode escape");
                        }
                        append_utf8(out, code);
                    } else throw std::runtime_error("Truncated unicode escape");
                    break;
                }
                default:
                    out += next;
            }
        } else out += c;
    }
    return out;
}

int main(int argc, char *argv[]) {
    bool raw = false;
    bool nixOnly = false;
    std::vector<std::string> positional;

    if (argc < 2) {
        print_usage(argv[0]);
    }

    // Parse flags and collect positional args
    for (int i = 1; i < argc; ++i) {
        std::string arg = argv[i];
        if (arg == "--raw") {
            raw = true;
        } else if (arg == "--nix") {
            nixOnly = true;
        } else if (arg == "help" || arg == "--help" || arg == "-h") {
            print_usage(argv[0]);
        } else {
            positional.push_back(arg);
        }
    }

    // Require at least the Nix expression
    if (positional.empty()) {
        std::cerr << "Error: Missing <nix_expr>.\n";
        print_usage(argv[0]);
    }

    std::string codeExpr = positional[0];
    bool hasFile = positional.size() > 1;
    std::string filePath;
    if (hasFile) filePath = positional[1];

    // Build input expression, respecting --nix flag
    std::string inputExpr;
    if (nixOnly) {
        inputExpr = "null";
    } else if (hasFile) {
        if (!filePath.empty()) {
            for (char &c : filePath) if (c == '\\') c = '/';
            inputExpr = "builtins.fromJSON (builtins.readFile " + filePath + ")";
        } else {
            inputExpr = "builtins.fromJSON (" + nix_string_literal("") + ")";
        }
    } else {
        std::string json = slurp_stdin();
        inputExpr = "builtins.fromJSON (" + nix_string_literal(json) + ")";
    }

    // Evaluate
    std::string fullCode = "with builtins; " + codeExpr;
    char *rawRes = eval_nix_expr(inputExpr.c_str(), fullCode.c_str());
    if (!rawRes) {
        std::cerr << "Evaluation failed or returned null." << std::endl;
        return EXIT_FAILURE;
    }
    std::string result(rawRes);
    free_cstring(rawRes);

    if (raw) {
        if (result.size() >= 2 && result.front() == '"' && result.back() == '"')
            result = result.substr(1, result.size() - 2);
        std::cout << unescape_json(result);
    } else {
        std::cout << result;
    }

    return EXIT_SUCCESS;
}
