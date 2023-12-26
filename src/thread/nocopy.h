# ifndef __NOCOPY_H__
# define __NOCOPY_H__

namespace FunDB {

class NoCopy {
public:
    NoCopy() = default;
    NoCopy(const NoCopy&) = delete;
    NoCopy& operator=(const NoCopy&) = delete;
};

} // namespace FunDB


# endif