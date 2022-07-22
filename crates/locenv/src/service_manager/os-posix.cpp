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
#include <sys/select.h>
#include <unistd.h>

static bool terminating;

static void handle_signal(int)
{
    terminating = true;
}

extern "C" void enter_daemon(const char *log)
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
}

extern "C" uint8_t event_loop(int server, event_loop_handler_t handler, void *context)
{
    // Block all signals.
    sigset_t mask;

    sigfillset(&mask);

    if (sigprocmask(SIG_SETMASK, &mask, nullptr) < 0) {
        auto c = errno;
        std::cout << "Failed to block signals: " << strerror(c) << std::endl;
        return BLOCK_SIGNALS_FAILED;
    }

    // Set SIGTERM handler.
    struct sigaction act;

    memset(&act, 0, sizeof(act));

    act.sa_handler = handle_signal;

    if (sigaction(SIGTERM, &act, nullptr) < 0) {
        auto c = errno;
        std::cout << "Failed to install SIGTERM handler: " << strerror(c) << std::endl;
        return SIGACTION_FAILED;
    }

    // Enter event loop.
    int max = server + 1;
    fd_set descriptors;

    FD_ZERO(&descriptors);
    FD_SET(server, &descriptors);

    sigdelset(&mask, SIGTERM);

    for (;;) {
        // Wait for events.
        fd_set ready;
        int remaining;

        memcpy(&ready, &descriptors, sizeof(fd_set));

        if ((remaining = pselect(max, &ready, nullptr, nullptr, nullptr, &mask)) < 0) {
            auto c = errno;

            if (c == EINTR) {
                if (terminating) {
                    return SUCCESS;
                }

                continue;
            }

            std::cout << "Failed to wait for events: " << strerror(c) << std::endl;
            return SELECT_FAILED;
        }

        // Process event.
        for (int fd = 0; remaining && fd < max; fd++) {
            uint8_t res = 0;

            if (!FD_ISSET(fd, &ready)) {
                continue;
            }

            if (fd == server) {
                res = handler(context, LOCENV_CLIENT_CONNECT, reinterpret_cast<const void *>(fd));
            }

            if (res) {
                return res;
            }

            remaining--;
        }
    }
}
