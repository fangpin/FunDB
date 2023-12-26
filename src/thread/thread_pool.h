#ifndef  __THREAD_POOL_H__
#define  __THREAD_POOL_H__

#include <vector>
#include <thread>
#include <queue>
#include <future>
#include "nocopy.h"

namespace FunDB {

class ThreadPool : public NoCopy {
    public:
        ThreadPool(int nThreads);
        ~ThreadPool() {
            Join();
        }

        template<typename Func, typename... Args>
        void AddTask(Func&& f, Args&&... params);
        void Join();

        bool IsStopped() {
            std::lock_guard<std::mutex> lock(mtx_);
            return stoped_;
        }

    private:
        std::vector<std::thread> threads_;
        std::queue<std::packaged_task<void()>> tasks_;
        std::mutex mtx_;
        std::condition_variable cv_;
        bool stoped_;
};

} // namespace FunDB

#endif
