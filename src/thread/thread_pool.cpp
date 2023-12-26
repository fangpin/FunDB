#include "thread_pool.h"

namespace FunDB {
    ThreadPool::ThreadPool(int nThreads) : stoped_(false) {
        for (size_t i = 0; i < nThreads; ++i)  {
            threads_.emplace_back([&](){
                for (;;) {
                    std::unique_lock lock(mtx_);
                    if (tasks_.empty()) {
                        if (stoped_) {
                            break;
                        }
                        cv_.wait(lock, [&](){
                            return tasks_.empty();
                        });
                    }
                    auto task = std::move(tasks_.front());
                    tasks_.pop();
                    task();
                }
            });
        }
    }

    template<typename Func, typename... Args>
    void ThreadPool::AddTask(Func&& f, Args&&... params) {
        assert(!stoped_);
        std::lock_guard lock(mtx_);
        bool empty = tasks_.empty();
        tasks_.emplace(std::forward(f), std::forward(params...));
        if (empty) {
            cv_.notify_all();
        }
    }

    void ThreadPool::Join() {
        std::unique_lock lock(mtx_);
        stoped_ = true;
        for (auto & t: threads_) {
            t.join();
        }
    }
} // namespace FunDB
