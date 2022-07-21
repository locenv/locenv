#include <iostream>
#include <ostream>
#include <sstream>
#include <stdexcept>

#include <errno.h>
#include <fcntl.h>
#include <inttypes.h>
#include <signal.h>
#include <string.h>
#include <sys/select.h>
#include <unistd.h>

extern "C" {
    extern const uint8_t BLOCK_SIGNALS_FAILED;
    extern const uint8_t SELECT_FAILED;
}

extern "C" void redirect_console_output(const char *file)
{
    // Create a file.
    auto fd = open(file, O_CREAT | O_WRONLY | O_TRUNC | O_CLOEXEC, S_IRUSR | S_IWUSR | S_IRGRP | S_IROTH);

    if (fd < 0) {
        auto c = errno;
        std::stringstream m;

        m << "Cannot open " << file << ": " << strerror(c);

        throw std::runtime_error(m.str());
    }

    // Duplicate to STDOUT.
    if (dup2(fd, STDOUT_FILENO) < 0) {
        auto c = errno;
        std::stringstream m;

        m << "Failed to set stdout to " << file << ": " << strerror(c);

        throw std::runtime_error(m.str());
    }

    // Duplicate it to STDERR.
    if (dup2(fd, STDERR_FILENO) < 0) {
        auto c = errno;
        std::stringstream m;

        m << "Failed to set stderr to " << file << ": " << strerror(c);

        throw std::runtime_error(m.str());
    }

    // Close the original FD.
    close(fd);
}

extern "C" uint8_t event_loop(int server)
{
    // Block all signals.
    sigset_t mask;

    sigfillset(&mask);

    if (sigprocmask(SIG_SETMASK, &mask, nullptr) < 0) {
        auto c = errno;
        std::cout << "Failed to block signals: " << strerror(c) << std::endl;
        return BLOCK_SIGNALS_FAILED;
    }

    // Enter event loop.
    sigdelset(&mask, SIGTERM);

    for (;;) {
        // Wait for events.
        if (pselect(1, nullptr, nullptr, nullptr, nullptr, &mask) < 0) {
            auto c = errno;

            if (c == EINTR) {
                continue;
            }

            std::cout << "Failed to wait for events: " << strerror(c) << std::endl;
            return SELECT_FAILED;
        }
    }
}
