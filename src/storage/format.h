#ifndef _FORMAT_H_
#define _FORMAT_H_

#include "util/status.h"
#include <cstdint>

namespace FunDB {

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

  Status DecodeFrom(const char *buf, EntryHeader &header);

  uint64_t EncodeTo(char *buf) const;
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
