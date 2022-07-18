#include <sstream>
#include <stdexcept>

#include <errno.h>
#include <fcntl.h>
#include <string.h>
#include <unistd.h>

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
