#pragma once
#include <string>
#include <unordered_set>
#include <unordered_map>
#include <thread>
#include <mutex>
#include <chrono>
#include <atomic>
#include "Command.h"

class Worker {
public:
    Worker();
    ~Worker();

    void Start();
    void Stop();

private:

    // ===== State =====
    std::atomic<bool> _running;
    std::chrono::steady_clock::time_point _sessionStart;
    std::chrono::steady_clock::time_point _lastSyncTime;
    std::string _currentProcessName;
    std::string _currentWindowTitle;

    // Visible apps tracking
    std::unordered_set<std::string> _currentlyVisibleApps;
    std::unordered_map<std::string, std::chrono::steady_clock::time_point> _visibleStartTimes;
    std::unordered_map<std::string, std::string> _lastWindowTitles;
    std::unordered_map<std::string, std::string> _visibleProcName;
    std::mutex _visibleAppsLock;

    // Blocked apps
    std::unordered_set<std::string> _blockedApps;
    std::mutex _blockLock;

    // Suppression window (prevent immediate re-kill after relaunch)
    std::unordered_map<std::string, std::chrono::steady_clock::time_point> _suppressKillUntil;
    std::mutex _suppressLock;

    // Command deduplication & State
    std::unordered_set<int> _appliedCommandIds;
    std::unordered_map<std::string, int> _lastAppliedCmdId; // Dedup per app (as per user snippet)
    std::mutex _cmdLock;

    // ===== App Usage Limits & Tracking =====
    std::unordered_map<std::string, int> _appUsageSeconds;      // usage today
    std::unordered_map<std::string, int> _appLimitsSeconds;     // limit (0 = no limit)
    std::unordered_set<std::string> _limitOverriddenToday;      // apps relaunched after limit hit
    std::string _lastUsageDate;                                 // YYYY-MM-DD
    std::mutex _usageLock;

    void LoadUsageStats();
    void SaveUsageStats();
    void CheckDailyReset(); // reset usage if new day
    void SyncDailyUsage();  // fetch usage from backend

    // ===== Threads =====
    std::thread _focusThread;
    std::thread _screenThread;
    std::thread _pollThread;
    std::thread _enforceThread;
    std::thread _liveThread;
    std::thread _zmqThread; // New ZMQ thread

    std::thread _locationThread;
    // ...
    void LocationReceiverLoop();

    // ===== Loop Functions =====
    void FocusLoop();
    void ScreenTimeLoop();
    void PollCommandsLoop();
    void EnforceLoop();
    void LiveStatusLoop();
    void ZmqLoop(); // New ZMQ loop

    // ===== Helpers =====
    void EndCurrentSession();
    void TrackVisibleWindows();
    void HandleCommand(const Command& cmd);
    void KillApp(const std::string& appName);
    void LaunchApp(const std::string& appName);
    void SendActivityLog(const std::string& body);
    void SendLiveStatus();
    void SendAlert(const std::string& body);
    std::string GetCurrentTimestamp();
    std::string DetectWebApp(const std::string& windowTitle);
    std::string MachineName();
};
