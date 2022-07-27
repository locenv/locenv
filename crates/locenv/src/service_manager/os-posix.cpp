#include "os.hpp"

#include <iostream>
#include <ostream>
#include <sstream>
#include <stdexcept>

#include <errno.h>
#include <fcntl.h>
#include <signal.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

static int terminating;

static void handle_signal(int)
{
    terminating = 1;
}

extern "C" int is_shutdown_requested() {
    return terminating;
}

extern "C" uint8_t enter_daemon(const char *log, uint8_t (*daemon) (void *), void *context)
{
    // Create a log file.
    auto fd = open(log, O_CREAT | O_WRONLY | O_TRUNC, S_IRUSR | S_IWUSR | S_IRGRP | S_IROTH);

    if (fd < 0) {
        auto c = errno;
        std::stringstream m;

        m << "Cannot open " << log << ": " << strerror(c);

        throw std::runtime_error(m.str());
    }

    // Duplicate to STDOUT.
    if (dup2(fd, STDOUT_FILENO) < 0) {
        auto c = errno;
        std::stringstream m;

        m << "Failed to set stdout to " << log << ": " << strerror(c);

        throw std::runtime_error(m.str());
    }

    // Duplicate it to STDERR.
    if (dup2(fd, STDERR_FILENO) < 0) {
        auto c = errno;
        std::stringstream m;

        m << "Failed to set stderr to " << log << ": " << strerror(c);

        throw std::runtime_error(m.str());
    }

    // Close the original FD.
    close(fd);

    // Create a new session and become a session leader.
    if (setsid() < 0) {
        auto c = errno;
        std::stringstream m;

        m << "Start new session failed: " << strerror(c);

        throw std::runtime_error(m.str());
    }

    // Kill the current process so we are not running as a session leader so we cannot accidentally acquire a controlling terminal.
    switch (fork()) {
    case -1: // Error.
        {
            auto c = errno;
            std::stringstream m;

            m << "Fork failed: " << strerror(c);

            throw std::runtime_error(m.str());
        }
        break;
    case 0: // We are in the child.
        break;
    default: // We are in the parent.
        // Use exit instead of return otherwise Rust object will get destructed.
        exit(SUCCESS);
    }

    // Block all signals.
    sigset_t mask;

    sigfillset(&mask);

    if (sigprocmask(SIG_SETMASK, &mask, nullptr) < 0) {
        auto c = errno;
        std::stringstream m;

        m << "Failed to block signals: " << strerror(c);

        throw new std::runtime_error(m.str());
    }

    // Set SIGTERM handler.
    struct sigaction act;

    memset(&act, 0, sizeof(act));

    act.sa_handler = handle_signal;

    if (sigaction(SIGTERM, &act, nullptr) < 0) {
        auto c = errno;
        std::stringstream m;

        m << "Failed to install SIGTERM handler: " << strerror(c);

        throw new std::runtime_error(m.str());
    }

    return daemon(context);
}
