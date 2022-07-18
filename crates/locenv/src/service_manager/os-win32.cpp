#include <memory>
#include <sstream>
#include <stdexcept>

#include <string.h>
#include <windows.h>

static std::unique_ptr<wchar_t[]> from_utf8(const char *utf8)
{
    // Get buffer size.
    auto bytes = (int)strlen(utf8) + 1;
    auto required = MultiByteToWideChar(CP_UTF8, 0, utf8, bytes, nullptr, 0);

    if (!required) {
        auto code = GetLastError();
        std::stringstream message;

        message << "Cannot get a length for buffer to decode " << utf8 << " (" << code << ")";

        throw std::runtime_error(message.str());
    }

    // Decode.
    auto buffer = std::make_unique<wchar_t[]>(required);

    if (!MultiByteToWideChar(CP_UTF8, 0, utf8, bytes, buffer.get(), required)) {
        auto code = GetLastError();
        std::stringstream message;

        message << "Cannot decode " << utf8 << " (" << code << ")";

        throw std::runtime_error(message.str());
    }

    return buffer;
}

extern "C" void redirect_console_output(const char *file)
{
    // Create file.
    auto name = from_utf8(file);
    auto handle = CreateFileW(name.get(), GENERIC_WRITE, FILE_SHARE_READ, nullptr, CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, nullptr);

    if (handle == INVALID_HANDLE_VALUE) {
        auto c = GetLastError();
        std::stringstream m;

        m << "Cannot create " << file << " (" << c << ")";

        throw std::runtime_error(m.str());
    }

    // Set STDOUT.
    if (!SetStdHandle(STD_OUTPUT_HANDLE, handle)) {
        auto c = GetLastError();
        std::stringstream m;

        m << "Cannot use " << file << " as a standard output device (" << c << ")";

        throw std::runtime_error(m.str());
    }

    // Set STDERR.
    if (!SetStdHandle(STD_ERROR_HANDLE, handle)) {
        auto c = GetLastError();
        std::stringstream m;

        m << "Cannot use " << file << " as a standard error device (" << c << ")";

        throw std::runtime_error(m.str());
    }
}
