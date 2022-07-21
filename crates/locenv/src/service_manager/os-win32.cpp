#include <iostream>
#include <memory>
#include <ostream>
#include <sstream>
#include <stdexcept>

#include <stdint.h>
#include <string.h>
#include <windows.h>

extern "C" {
    extern const uint8_t SUCCESS;
    extern const uint8_t CREATE_WINDOW_FAILED;
    extern const uint8_t REGISTER_CLASS_FAILED;
    extern const uint8_t EVENT_LOOP_FAILED;
}

static LRESULT message_proc(HWND hWnd, UINT uMsg, WPARAM wParam, LPARAM lParam)
{
    return 0;
}

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

extern "C" uint8_t event_loop(SOCKET server)
{
    // Create message-only window.
    WNDCLASSEXW wc = {0};

    wc.cbSize = sizeof(wc);
    wc.lpfnWndProc = message_proc;
    wc.hInstance = GetModuleHandleW(nullptr);
    wc.lpszClassName = L"locenv-service-manager";

    auto atom = RegisterClassExW(&wc);

    if (!atom) {
        auto code = GetLastError();
        std::wcerr << L"Failed to register a window class (" << code << L")" << std::endl;
        return REGISTER_CLASS_FAILED;
    }

    auto wnd = CreateWindowExW(0, (LPCWSTR)atom, nullptr, 0, 0, 0, 0, 0, HWND_MESSAGE, nullptr, GetModuleHandleW(nullptr), nullptr);

    if (!wnd) {
        auto code = GetLastError();
        std::wcerr << L"Failed to create a window (" << code << L")" << std::endl;
        return CREATE_WINDOW_FAILED;
    }

    // Process events until WM_QUIT.
    MSG msg;
    BOOL res;

    while ((res = GetMessage(&msg, nullptr, 0, 0)) != 0) {
        if (res == -1) {
            auto code = GetLastError();
            std::wcerr << L"Failed get a Windows message (" << code << L")" << std::endl;
            return EVENT_LOOP_FAILED;
        } else {
            TranslateMessage(&msg);
            DispatchMessage(&msg);
        }
    }

    return SUCCESS;
}
