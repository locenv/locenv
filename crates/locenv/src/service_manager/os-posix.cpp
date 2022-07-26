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

static sigset_t mask;
static int max_fd;
static fd_set readfds;
static bool terminating;

static void handle_signal(int)
{
    terminating = true;
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

    sigdelset(&mask, SIGTERM);

    return daemon(context);
}

extern "C" void register_for_accept(int fd)
{
    if (fd >= max_fd) {
        max_fd = fd + 1;
    }

    FD_SET(fd, &readfds);
}

extern "C" uint8_t dispatch_events(void (*handler) (int))
{
    if (!max_fd) {
        return NO_EVENT_SOURCES;
    }

    for (;;) {
        // Wait for events.
        fd_set readfds;
        int remaining;

        memcpy(&readfds, &::readfds, sizeof(fd_set));

        if ((remaining = pselect(max_fd, &readfds, nullptr, nullptr, nullptr, &mask)) < 0) {
            auto c = errno;

            if (c == EINTR) {
                if (terminating) {
                    return DISPATCHER_TERMINATED;
                }

                continue;
            }

            std::cout << "Failed to wait for events: " << strerror(c) << std::endl;
            return SELECT_FAILED;
        }

        // Invoke handler.
        for (int fd = 0; remaining && fd < max_fd; fd++) {
            if (FD_ISSET(fd, &readfds)) {
                FD_CLR(fd, &::readfds);
            } else {
                continue;
            }

            handler(fd);
            remaining--;
        }

        return SUCCESS;
    }
}
