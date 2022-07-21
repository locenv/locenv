#ifndef LOCENV_OS_HPP_INCLUDED
#define LOCENV_OS_HPP_INCLUDED

#ifdef _MSC_VER
#include <stdint.h>
#else
#include <inttypes.h>
#endif

#define LOCENV_CLIENT_CONNECT 0

extern "C" {
    extern const uint8_t SUCCESS;
    extern const uint8_t SIGACTION_FAILED;
    extern const uint8_t WAIT_CLIENT_FAILED;
    extern const uint8_t GET_WINDOW_LONG_FAILED;
    extern const uint8_t SET_WINDOW_LONG_FAILED;
    extern const uint8_t ASYNC_SELECT_FAILED;
    extern const uint8_t BLOCK_SIGNALS_FAILED;
    extern const uint8_t SELECT_FAILED;
    extern const uint8_t CREATE_WINDOW_FAILED;
    extern const uint8_t REGISTER_CLASS_FAILED;
    extern const uint8_t EVENT_LOOP_FAILED;
}

typedef uint8_t (*event_loop_handler_t) (void *context, uint32_t event, const void *data);

#endif // LOCENV_OS_HPP_INCLUDED
