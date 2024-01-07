#include "format.h"
#include "util/status.h"

#include <cstring>

namespace FunDB {

template <std::integral T> T decode(const char *buf) {
  T ret;
  std::memcpy(&ret, buf, sizeof(ret));
  return ret;
}

template <std::integral T> void encode(T const &val, char *buf) {
  std::memcpy(buf, &val, sizeof(val));
}

struct EntryType {
  const static uint32_t Deleted = 1;
  const static uint32_t Compressed = (1 << 1);
};

bool EntryHeader::IsDeleted() const { return flags_ & EntryType::Deleted; }

bool EntryHeader::IsCompressed() const {
  return flags_ & EntryType::Compressed;
}

Status EntryHeader::DecodeFrom(const char *buf, EntryHeader &header) {
  char *p = const_cast<char *>(buf);

  header.checksum_ = decode<decltype(header.checksum_)>(p);
  p += sizeof(header.checksum_);

  header.flags_ = decode<decltype(header.flags_)>(p);
  p += sizeof(header.flags_);

  header.sizeKey_ = decode<decltype(header.sizeKey_)>(p);
  p += sizeof(header.sizeKey_);

  header.sizeValue_ = decode<decltype(header.sizeKey_)>(p);
  p += sizeof(header.sizeKey_);

  header.sizeValueCompressed_ =
      decode<decltype(header.sizeValueCompressed_)>(p);
  p += sizeof(header.sizeValueCompressed_);

  header.hash_ = decode<decltype(header.hash_)>(p);
  p += sizeof(header.hash_);

  return Status{};
}

uint64_t EntryHeader::EncodeTo(char *buf) const {
  encode(checksum_, buf);
  encode(flags_, buf);
  encode(sizeKey_, buf);
  encode(sizeValue_, buf);
  encode(sizeValueCompressed_, buf);
  encode(hash_, buf);
  return 0;
}

} // namespace FunDB
