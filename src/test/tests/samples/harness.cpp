/*
 * Copyright 2023 Austin Adams. No rights reserved. Any reproduction or distribution without the prior written consent of Austin Adams is fine because he does not give a shit.
 */

// This is a handy little script for running a LC-3 object file with getc
// hooked up to stdin and putc hooked up to stdout. That way you can use it
// inside shell pipelines in the host OS.
//
// You'd think you could do that with the `simulator' binary, but I encourage
// you to try that if you have good health insurance. I have been on blood
// pressure mediation ever since EOF sent the stock CLI simulator into an
// infinite loop.

#include <iostream>
#include <cstring>
#include <fcntl.h>
#include <unistd.h>

#include "console_printer.h"
#include "interface.h"
#include "inputter.h"

namespace {
    // Similar to lc3::ConsoleInputter except not overcomplicated by imitating
    // the Windows API for no apparent reason. Will not work on Windows but I
    // don't give a fuck
    class StdinInputter : public lc3::utils::IInputter
    {
    #define BUF_SIZE 1024
    #define MAX_EOFS 32

    public:
        StdinInputter(void) = default;
        ~StdinInputter(void) = default;

        virtual bool hasRemaining(void) const override {
            return len && off < len;
        }

        virtual void beginInput(void) override {
            old_stdin_flags = fcntl(0, F_GETFL);
            fcntl(0, F_SETFL, old_stdin_flags | O_NONBLOCK);

            eofs = 0;
            off = 0;
            len = 0;
        }

        virtual void endInput(void) override {
            // Per this grumpy guy, this may be useful:
            // https://stackoverflow.com/questions/717572/how-do-you-do-non-blocking-console-i-o-on-linux-in-c#comment120912307_30548692
            fcntl(0, F_SETFL, old_stdin_flags);
        }

        virtual bool getChar(char & c) override {
            ssize_t ret;
            // Attempt some humble buffering to avoid a gajillion syscalls
            if (len && off < len) {
                c = buf[off++];
                return true;
            } else if ((ret = read(0, buf, BUF_SIZE)) < 0) {
                if (errno == EAGAIN || errno == EWOULDBLOCK) {
                    return false;
                } else {
                    throw lc3::utils::exception("read() failed: " + std::string(std::strerror(errno)));
                }
            } else if (!ret) {
                // What is this crap? Good question! It turns out that Mr.
                // Chirag will repeatedly call getChar() and add it to his
                // buffer of characters until the cows come home. Thus if we
                // always return EOT (or whatever) on EOF, we will fill up
                // memory with EOTs. It's tempting to return EOT once, but on
                // real Unix, I can call fgetc() and get EOF as many times as I
                // want. So compromise with a hack: add a healthy number (say,
                // 32) EOTs to Chirag's buffer, and after that, pretty much
                // deadlock. That's the best we can do.
                if (eofs++ < MAX_EOFS) {
                    // For EOF, return the EOT character. Not perfect since actual
                    // EOTs will be false positives, but there's not much else we
                    // can do.
                    c = '\x04';
                    return true;
                } else {
                    return false;
                }
            } else { // ret > 0
                c = buf[0];
                off = 1;
                len = (size_t)ret;
                return true;
            }
        }

    private:
        int eofs;
        int old_stdin_flags;
        char buf[BUF_SIZE];
        size_t off;
        size_t len;
    };

    // Similar to lc3::ConsolePrinter except we don't spam write() syscalls for
    // no reason
    class StdoutPrinter : public lc3::utils::IPrinter
    {
    public:
        virtual void setColor(lc3::utils::PrintColor color) override { /* Pound sand */ }
        virtual void print(std::string const & string) override {
            // Note we do not flush here, unlike Chirag's ConsolePrinter. In our
            // use case, there is no need to flush for every character; that's
            // just wasteful.
            std::cout << string;
        }
        virtual void newline(void) override {
            std::cout << "\n";
        }
    };
};

int main(int argc, char **argv) {
    if (argc-1 != 1) {
        std::cerr << "usage: " << argv[0] << " [obj file]\n";
        return 1;
    }

    std::string objfilename(argv[1]);

    StdoutPrinter printer;
    StdinInputter inputter;
    //uint32_t print_level = DEFAULT_PRINT_LEVEL;
    uint32_t print_level = 4;
    lc3::sim simulator(printer, inputter, print_level);
    simulator.loadObjFile(objfilename);
    simulator.run();

    return 0;
}
