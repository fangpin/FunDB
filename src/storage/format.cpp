#include "format.h"

namespace FunDB {

bool EntryHeader::IsDeleted() const {
    return flags_ & EntryType::Deleted;
}

bool EntryHeader::IsCompressed() const {
    return flags_ & EntryType::Compressed;
}

// todo: implement me
Status EntryHeader::DecodeFrom(const char* buf, EntryHeader const& header) {
    return Status{};
}

// todo: implement me
uint64_t EntryHeader::EncodeTo(char* buf) const {
    return 0;
}

} // namespace FunDB