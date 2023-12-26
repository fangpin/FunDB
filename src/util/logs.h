#ifndef _LOGS_H_
#define _LOGS_H_

#include <string_view>
#include <iostream>
#include <cstdio>

namespace FunDB {
class Log
{
    static inline void Debug(std::string_view msg) {
        std::cout << "[Debug] [" << __FILE__  << "] " << __LINE__ << " ]: " << msg << std::endl;
    }

    static inline void Info(std::string_view msg) {
        std::cout << "[Info] [" << __FILE__  << "] " << __LINE__ << " ]: " << msg << std::endl;
    }

    static inline void Warn(std::string_view msg) {
        std::cout << "[Warn] [" << __FILE__  << "] " << __LINE__ << " ]: " << msg << std::endl;
    }

    static inline void Error(std::string_view msg) {
        std::cout << "[Error] [" << __FILE__  << "] " << __LINE__ << " ]: " << msg << std::endl;
    }
};
} // namespace FunDB

#endif