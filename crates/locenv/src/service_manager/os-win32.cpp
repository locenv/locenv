#include "os.hpp"

#include <iostream>
#include <memory>
#include <ostream>
#include <sstream>
#include <stdexcept>

#include <string.h>
#include <windows.h>
#include <winsock2.h>

#define WM_RPC_SERVER (WM_USER + 0)

struct event_handler {
    event_loop_handler_t handler;
    void *context;
};

static std::unique_ptr<wchar_t[]> from_utf8(const char *utf8)
{
    // Get buffer size.
    auto bytes = static_cast<int>(strlen(utf8) + 1);
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

static LRESULT message_proc(HWND hWnd, UINT uMsg, WPARAM wParam, LPARAM lParam)
{
    auto handler = reinterpret_cast<const event_handler *>(GetWindowLongPtrW(hWnd, GWLP_USERDATA));

    if (!handler) {
        auto code = GetLastError();
        std::wcerr << L"Failed to get user data from the window (" << code << L")" << std::endl;
        PostQuitMessage(GET_WINDOW_LONG_FAILED);
        return 0;
    }

    switch (uMsg) {
    case WM_RPC_SERVER:
        {
            auto server = reinterpret_cast<SOCKET>(wParam);
            auto error = WSAGETSELECTERROR(lParam);

            if (error) {
                PostQuitMessage(WAIT_CLIENT_FAILED);
                std::wcerr << L"An error occurred while waiting for a connection from RPC client (" << error << L")" << std::endl;
                return 0;
            }

            auto result = handler->handler(handler->context, LOCENV_CLIENT_CONNECT, reinterpret_cast<const void *>(server));

            if (result) {
                PostQuitMessage(result);
            }
        }
        break;
    }

    return 0;
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

extern "C" uint8_t event_loop(SOCKET server, event_loop_handler_t handler, void *context)
{
    // Create a message-only window to receive event from Windows.
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

    // Associate the handler with the window.
    event_handler ud = {0};

    ud.handler = handler;
    ud.context = context;

    SetLastError(0);

    if (!SetWindowLongPtrW(wnd, GWLP_USERDATA, reinterpret_cast<LONG_PTR>(&ud))) {
        auto code = GetLastError();

        if (code) {
            std::wcerr << L"Failed to associate an event handler with the window (" << code << L")" << std::endl;
            return SET_WINDOW_LONG_FAILED;
        }
    }

    // Listen for notification on RPC server.
    if (WSAAsyncSelect(server, wnd, WM_RPC_SERVER, FD_ACCEPT) != 0) {
        auto code = WSAGetLastError();
        std::wcerr << L"Failed to listen for notification on RPC server (" << code << L")" << std::endl;
        return ASYNC_SELECT_FAILED;
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

    return static_cast<uint8_t>(msg.wParam);
}
