#ifndef _STATUS_H_
#define _STATUS_H_

#include <string>
#include <string_view>

namespace FunDB {

class Status{
public:
    enum class Code {
        OK,
        Invalid,
    };

    Status(Status::Code code = Code::OK, std::string_view msg = "") : code_(Code::OK), msg_(msg) {}

    ~Status(){}

    bool OK() const {
        return code_ == Code::OK;
    }

    std::string const& Message() const {
        return msg_;
    }

private:
    Code code_;
    std::string msg_;
};

} // namespace FunDB

#endif