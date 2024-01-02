#ifndef _FORMAT_H_
#define _FORMAT_H_

#include <cstdint>
#include <cstring>
#include "util/status.h"

namespace FunDB {

static uint32_t decodeU32(const char* buf) {
    uint32_t ret;
    std::memcpy(&ret, buf, sizeof(ret));
    return ret;
}

static void encodeU32(char* buf, uint32_t val) {
    std::memcpy(buf, &val, sizeof(val));
}

static uint64_t decodeU64(const char* buf) {
    uint64_t ret;
    std::memcpy(&ret, buf, sizeof(ret));
    return ret;
}

static void encodeU64(char* buf, uint64_t val) {
    std::memcpy(buf, &val, sizeof(val));
}

struct EntryType {
    const static uint32_t Deleted = 1;
    const static uint32_t Compressed = (1 << 1);
};

struct EntryHeader {
    public:
    uint32_t checksum_;
    uint32_t flags_;
    uint64_t sizeKey_;
    uint64_t sizeValue_;
    uint64_t sizeValueCompressed_;
    uint64_t hash_;

    bool IsDeleted() const;

    bool IsCompressed() const;

    Status DecodeFrom(const char* buf, EntryHeader const& header);

    uint64_t EncodeTo(char* buf) const;
};

struct TableHeader {
    uint32_t checksum_;
    uint32_t majorVersion_;
    uint32_t minorVersion_;
    uint32_t revisionVersion_;
    uint32_t buildVersion_;

    uint32_t majorDataFormatVersion_;
    uint32_t minorDataFormatVersion_;
};

struct OffsetArray {
    uint64_t hash_;
    uint64_t offset_;
};

} // namespace FunDB

#endif