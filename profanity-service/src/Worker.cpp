#include "Worker.h"
#include <zmq.hpp>
#include "HttpClient.h"
#include "WinUtils.h"
#include "json.hpp"
#include <iostream>
#include <tlhelp32.h>
#include <sstream>
#include <algorithm>
#include <unordered_map>
#include <unordered_set>
#include <iomanip>
#include <fstream>
#include <windows.h>
#include <iphlpapi.h> 
#pragma comment(lib, "iphlpapi.lib")

// ======================================================
// Shared HTTP client instance (token lifecycle lives here)
// ======================================================
static HttpClient g_httpClient;


using json = nlohmann::json;

static std::string deviceUuid;
static std::string macAddress;
static int deviceId = 0;           // returned by backend after registration
static int childId = 0;            // optional — backend can assign

// ======================================================
// Web App Patterns (MOVE TO TOP)
// ======================================================

std::string GetPrimaryMAC();
std::string LookupUUIDByMAC();
static const std::unordered_map<std::string, std::vector<std::string>> WebAppPatterns = {
    {"google-chat", {"google chat", "chat.google.com", "chat -"}},
    {"whatsapp", {"whatsapp", "web.whatsapp.com"}},
    {"youtube", {"youtube", "youtube.com", " - youtube"}},
    {"gmail", {"gmail", "mail.google.com"}},
    {"postman", {"postman", "web.postman.co"}}};

// ======================================================
// Utilities
// ======================================================

static std::string SafeGetStr(const json& j, const std::string& key, const std::string& defaultValue = "") {
    if (j.is_object() && j.contains(key) && j[key].is_string()) {
        return j[key].get<std::string>();
    }
    return defaultValue;
}

static int SafeGetInt(const json& j, const std::string& key, int defaultValue = 0) {
    if (j.is_object() && j.contains(key)) {
        if (j[key].is_number_integer()) return j[key].get<int>();
        if (j[key].is_string()) {
            try { return std::stoi(j[key].get<std::string>()); } catch (...) {}
        }
    }
    return defaultValue;
}

std::string Worker::MachineName()
{
    wchar_t name[256];
    DWORD size = 256;
    GetComputerNameW(name, &size);

    std::wstring w(name);
    int len = WideCharToMultiByte(CP_UTF8, 0, w.c_str(), -1, nullptr, 0, nullptr, nullptr);
    std::string out(len, 0);
    WideCharToMultiByte(CP_UTF8, 0, w.c_str(), -1, &out[0], len, nullptr, nullptr);
    if (!out.empty() && out.back() == '\0')
        out.pop_back();

    return out;
}

static std::string ToLower(std::string s)
{
    std::transform(s.begin(), s.end(), s.begin(), ::tolower);
    return s;
}

// ======================================================
// DetectWebApp
// ======================================================
std::string Worker::DetectWebApp(const std::string &windowTitle)
{
    std::string t = windowTitle;
    std::transform(t.begin(), t.end(), t.begin(), ::tolower);

    for (const auto &kv : WebAppPatterns)
    {
        for (const auto &pattern : kv.second)
        {
            if (t.find(pattern) != std::string::npos)
                return kv.first;
        }
    }
    return "";
}

// ======================================================
// Constructor / Destructor
// ======================================================
Worker::Worker()
    : _running(false)
{
    _sessionStart = std::chrono::steady_clock::now();
    _lastSyncTime = std::chrono::steady_clock::now();
}
 
Worker::~Worker()
{
    Stop();
}

// ======================================================
// Sync Daily Usage with Backend (Source of Truth)
// ======================================================
void Worker::SyncDailyUsage()
{
    // 1. Check if we actually have any limits to enforce. If not, maybe skip to save bandwidth?
    //    User said "if limit-usage is active = true", but practically if we have limits in map, we should sync.
    bool hasLimits;
    {
        std::lock_guard<std::mutex> lock(_usageLock);
        hasLimits = !_appLimitsSeconds.empty();
    }
    
    // Allow syncing even if no limits yet, to keep dashboard in sync? 
    // Optimization: only sync if we are enforcing limits.
    if (!hasLimits) return;

    try
    {
        std::string today = _lastUsageDate; // e.g. "2026-02-03" initialized in LoadUsageStats
        // URL: /api/summary/daily-comparison?deviceId=...
        // User noted: "this end point can also call by deviceUuid or date"
        
        std::string url = "https://zyberhero.com/api/summary/daily-comparison?deviceId=" + std::to_string(deviceId);
        
        // std::cout << "[SYNC] Checking usage from backend: " << url << "\n";
        std::string res = g_httpClient.HttpGet(url);
        
        if (res.empty()) return;

        auto jsonRes = json::parse(res);
        
        // Response format is object with "apps" array
        if (jsonRes.contains("apps") && jsonRes["apps"].is_array())
        {
            std::lock_guard<std::mutex> lock(_usageLock);
            
            for (const auto& item : jsonRes["apps"])
            {
                std::string appName = SafeGetStr(item, "app");
                if (appName.empty()) continue;
                
                // Normalization: backend usually sends what we sent it (e.g. "chrome", "google-chat")
                // but let's ensure we match our local keys
                std::string key = ToLower(appName);
                
                int focused = SafeGetInt(item, "focusedTimeSeconds");
                int screen = SafeGetInt(item, "screenTimeSeconds");
                int totalBackend = focused + screen; // As per user instruction
                
                // Update local counter
                // We use MAX to never rewind local usage if backend is slightly behind
                // UNLESS totalBackend is significantly higher (which it should be if other sessions exist)
                
                int currentLocal = _appUsageSeconds[key];
                
                if (totalBackend > currentLocal)
                {
                    _appUsageSeconds[key] = totalBackend;
                    // std::cout << "[SYNC] Updated " << key << " usage to " << totalBackend << "s (was " << currentLocal << "s)\n";
                }
            }
        }
    }
    catch (const std::exception& e)
    {
        std::cerr << "[SYNC ERROR] Failed to sync usage: " << e.what() << "\n";
    }
}
const std::string UUID_PATH = "C:\\ProgramData\\ParentalMonitor\\device_uuid.txt";
void EnsureDirExists(const std::string& path) {
    std::wstring wpath(path.begin(), path.end());
    CreateDirectoryW(wpath.c_str(), NULL);
}

std::string LookupUUIDByMAC()
{
    std::string macAddress = GetPrimaryMAC();
    if (macAddress == "00:00:00:00:00:00") {
        throw std::runtime_error("Failed to capture a valid MAC address.");
    }

    std::cout << "[LOOKUP] Attempting to look up UUID for MAC: " << macAddress << std::endl;

    // Construct the API URL
    std::string baseUrl = "https://zyberhero.com";
    // std::string baseUrl = "http://localhost:8060";
    std::string apiPath = "/api/devices/uuid-by-mac";
    std::string url = baseUrl + apiPath + "?macAddress=" + macAddress;

    std::cout << "[LOOKUP] Full URL: " << url << "\n";

    // 1. Perform HTTP GET Request
    std::string responseBody = g_httpClient.HttpGet(url);

    std::cout << "[LOOKUP] Raw response: " << responseBody << "\n";

    if (responseBody.empty()) {
        throw std::runtime_error("Server lookup failed (Network Error or HTTP client failed).");
    }

    // --- JSON Parsing ---
    try {
        // 2. Parse the received string into a JSON object
        json jsonResponse = json::parse(responseBody);

        std::cout << "[LOOKUP] Parsed JSON. Keys: ";
        for (const auto& key : jsonResponse.items()) {
            std::cout << key.key() << " ";
        }
        std::cout << "\n";

        // 3. Check for required fields (deviceUuid + deviceId)
        // Accept ANY format that has these two fields
        bool hasUuid = jsonResponse.contains("deviceUuid");
        bool hasDeviceId = jsonResponse.contains("deviceId");

        std::cout << "[LOOKUP] Has deviceUuid: " << hasUuid << ", Has deviceId: " << hasDeviceId << "\n";

        if (hasUuid && hasDeviceId) {
            std::string uuid = SafeGetStr(jsonResponse, "deviceUuid");
            int devId = SafeGetInt(jsonResponse, "deviceId");
            int childId = SafeGetInt(jsonResponse, "childId");

            std::cout << "[LOOKUP SUCCESS] UUID: " << uuid << " | Device ID: " << devId << " | Child ID: " << childId << std::endl;

            // Return the full JSON response body for Worker::Start() to parse
            return responseBody;
        }

        // 4. If fields missing, check for error response
        if (jsonResponse.contains("error")) {
            std::string errorMessage = jsonResponse.value("error", "Unknown server error.");
            throw std::runtime_error("Device lookup failed: " + errorMessage);
        }

        // Catch-all for unexpected JSON structure
        throw std::runtime_error("Server response missing 'deviceUuid' or 'deviceId'. Got: " + responseBody.substr(0, 200));

    } catch (const json::exception& e) {
        std::cerr << "[LOOKUP ERROR] JSON Parsing Failed. Response (first 200 chars): " 
                  << responseBody.substr(0, 200) << "\n";
        throw std::runtime_error("JSON Parse Error: " + std::string(e.what()));
    }
}

// --- The rest of your MAC capture function remains the same ---
std::string GetPrimaryMAC() 
{
    // ... (Your implementation here)
    IP_ADAPTER_INFO adapterInfo[16];
    DWORD bufLen = sizeof(adapterInfo);

    if (GetAdaptersInfo(adapterInfo, &bufLen) == NO_ERROR)
    {
        PIP_ADAPTER_INFO pAdapter = adapterInfo;
        while (pAdapter)
        {
            if (pAdapter->Type == MIB_IF_TYPE_ETHERNET ||
                pAdapter->Type == IF_TYPE_IEEE80211)
            {
                char macStr[32];
                sprintf(macStr, "%02X:%02X:%02X:%02X:%02X:%02X",
                        pAdapter->Address[0], pAdapter->Address[1],
                        pAdapter->Address[2], pAdapter->Address[3],
                        pAdapter->Address[4], pAdapter->Address[5]);

                return std::string(macStr);
            }
            pAdapter = pAdapter->Next;
        }
    }
    return "00:00:00:00:00:00";
}
 

// ======================================================
// Start Threads
// ======================================================
// In Worker.cpp

// void Worker::Start()
// {
//     std::cout << "=============================\n";
//     std::cout << "[WORKER START] Called\n";
//     std::cout << "=============================\n";

//     std::cout << "[DEBUG] _running = " << _running << "\n";
//     if (_running) {
//         std::cout << "[DEBUG] Worker already running → skipping Start()\n";
//         return;
//     }
//     _running = true;

//     // ---------------------------
//     // Load identifiers (Unchanged)
//     // ---------------------------
//     // deviceUuid = LoadOrCreateUUID();
//     macAddress = GetPrimaryMAC();

//     std::cout << "[DEBUG] deviceUuid = " << deviceUuid << "\n";
//     std::cout << "[DEBUG] macAddress = " << macAddress << "\n";

//     // Build register JSON (Unchanged)
//     json reg;
//     reg["deviceUuid"] = deviceUuid;
//     reg["macAddress"] = macAddress;
//     reg["machineName"] = MachineName();
//     reg["userName"] = getenv("USERNAME") ? getenv("USERNAME") : "";
//     reg["os"] = WinUtils::GetOSVersion();
//     reg["childId"] = childId;

//     std::string body = reg.dump();
//     std::cout << "\n[REGISTER] Sending POST → http://localhost:3001/api/devices/update\n";
//     std::cout << "[REGISTER] Payload = " << body << "\n";

//     // ---------------------------
//     // Send registration request (Unchanged)
//     // ---------------------------
//     std::string response;

//     try {
//         response = HttpClient::HttpPost(
//             "http://localhost:3001/api/devices/update",
//             body
//         );

//         std::cout << "[REGISTER] Raw backend response: " << response << "\n";
//     }
//     catch (const std::exception &e) {
//         std::cerr << "[REGISTER ERROR] HttpPost exception: " << e.what() << "\n";
//         WinUtils::LogToFile("REGISTER ERROR HttpPost: " + std::string(e.what()));
//     }
//     catch (...) {
//         std::cerr << "[REGISTER ERROR] Unknown HttpPost exception\n";
//         WinUtils::LogToFile("REGISTER ERROR: Unknown HttpPost exception");
//     }

//     // ---------------------------
//     // Parse backend response (Unchanged)
//     // ---------------------------
//     try {
//         auto obj = json::parse(response);

//         deviceId = obj.value("deviceId", 0);
//         childId  = obj.value("childId", 0);

//         std::cout << "[REGISTER] Parsed deviceId = " << deviceId 
//                   << ", childId = " << childId << "\n";
//     }
//     catch (...) {
//         std::cerr << "[REGISTER ERROR] Failed to parse response JSON\n";
//         WinUtils::LogToFile("REGISTER ERROR: failed to parse JSON");
//     }

//     if (deviceId == 0) {
//         std::cerr << "\n❌ FATAL ERROR: Device registration failed.\n";
//         std::cerr << "   The agent CANNOT continue without a deviceId.\n";
//         WinUtils::LogToFile("FATAL: Registration failed → deviceId=0");
//         return;
//     }

//     std::cout << "\n[REGISTER SUCCESS] Device registered with ID = " << deviceId << "\n";

//     // ---------------------------
//     // Start all worker threads
//     // ---------------------------
//     _focusThread = std::thread(&Worker::FocusLoop, this);
//     _screenThread = std::thread(&Worker::ScreenTimeLoop, this);
//     _pollThread = std::thread(&Worker::PollCommandsLoop, this);
//     _enforceThread = std::thread(&Worker::EnforceLoop, this);
//     _liveThread = std::thread(&Worker::LiveStatusLoop, this);
    
//     // *** NEW: Start ZMQ PULL thread for Location Data on 5558 ***
//     // NOTE: Requires you to add 'std::thread _locationThread;' and 'void LocationReceiverLoop();' to Worker class definition
//    // *** ENHANCED LOGGING FOR LOCATION THREAD START ***
//     std::cout << "[INFO] Attempting to start Location Receiver (5558) thread...\n";
//     WinUtils::LogToFile("[INFO] Attempting to start Location Receiver (5558) thread...");
//     _locationThread = std::thread(&Worker::LocationReceiverLoop, this); 
    
//     _zmqThread = std::thread(&Worker::ZmqLoop, this);

//     std::cout << "[WORKER] All threads started.\n";
//     WinUtils::LogToFile("Worker started successfully");
// }

void Worker::Start()
{
    // ... (Initial setup and check for _running) ...
    
    _running = true;
    LoadUsageStats(); // Load persisted usage and limits

    // ---------------------------
    // 0. AUTHENTICATE
    // ---------------------------
    if (!g_httpClient.GetAccessToken()) {
        std::cerr << "\n❌ FATAL ERROR: Failed to obtain access token.\n";
        WinUtils::LogToFile("FATAL: Failed to obtain access token.");
        _running = false;
        return;
    }

    // ---------------------------
    // 1. CAPTURE MAC ADDRESS
    // ---------------------------
    macAddress = GetPrimaryMAC();
    std::cout << "[DEBUG] macAddress = " << macAddress << "\n";

    // ---------------------------
    // 2. ATTEMPT UUID LOOKUP (GET API)
    // ---------------------------
    bool registrationNeeded = false;
    std::string uuidResponse;

    try {
        // LookupUUIDByMAC returns the full JSON string on success
        uuidResponse = LookupUUIDByMAC(); 
        
        // --- Parse UUID Lookup Response to set IDs ---
        auto uuidObj = json::parse(uuidResponse); // Guaranteed to succeed due to try/catch in LookupUUIDByMAC

        // Extract and set IDs from the successful lookup response
        deviceId = SafeGetInt(uuidObj, "deviceId");
        childId = SafeGetInt(uuidObj, "childId");
        deviceUuid = SafeGetStr(uuidObj, "deviceUuid");

        std::cout << "[LOOKUP SUCCESS] Device ID set: " << deviceId << ", UUID: " << deviceUuid << "\n";

    } catch (const std::runtime_error &e) {
        // If LookupUUIDByMAC throws (not found, network error, parse error)
        std::cerr << "[LOOKUP ERROR] " << e.what() << "\n";
        registrationNeeded = true;
    }
    catch (...) {
        std::cerr << "[LOOKUP ERROR] Unknown exception during lookup.\n";
        registrationNeeded = true;
    }
    
    // Fallback: If deviceId is 0, we treat it as an unprovisioned device.
    if (deviceId == 0) {
        registrationNeeded = true;
    }

    // ---------------------------
    // 3. POST TO REGISTER/UPDATE IF NECESSARY
    // ---------------------------
    if (registrationNeeded) {
        std::cout << "\n[REGISTER] Device not provisioned. Sending POST to register/update.\n";

        // Generate a new UUID if we still don't have one 
        if (deviceUuid.empty()) {
           return;
        }

        json reg;
        reg["deviceUuid"] = deviceUuid;
        reg["macAddress"] = macAddress;
        reg["machineName"] = MachineName();
        reg["userName"] = getenv("USERNAME") ? getenv("USERNAME") : "";
        reg["os"] = WinUtils::GetOSVersion();
        reg["childId"] = childId; 

        std::string body = reg.dump();
        std::cout << "[REGISTER] Payload = " << body << "\n";

        std::string response;
        try {
            response = g_httpClient.HttpPost(
                "https://zyberhero.com/api/devices/register/update", 
                // "http://localhost:8060/api/devices/register/update",
                body
            );
            
            // CRITICAL FIX: The backend returns minimal JSON. 
            // We re-run Start() to force a new LookupUUIDByMAC call, which should now succeed.
            if (!response.empty() && response.find("\"success\":true") != std::string::npos) {
                 std::cout << "[REGISTER SUCCESS] POST accepted. Re-running Start() to fetch new IDs...\n";
                 return Start(); // Re-run the Start() function to re-attempt lookup
            }

            std::cerr << "[REGISTER ERROR] POST successful but final IDs acquisition failed.\n";
        
        } catch (const std::exception &e) {
            std::cerr << "[REGISTER ERROR] HttpPost exception: " << e.what() << "\n";
            WinUtils::LogToFile("REGISTER ERROR HttpPost: " + std::string(e.what()));
        }
        catch (...) {
            std::cerr << "[REGISTER ERROR] Unknown HttpPost exception\n";
            WinUtils::LogToFile("REGISTER ERROR: Unknown HttpPost exception");
        }
    }

    // ---------------------------
    // 4. CHECK FOR FATAL ERROR & START THREADS
    // ---------------------------
    if (deviceId == 0) {
        std::cerr << "\n❌ FATAL ERROR: Device registration/lookup failed.\n";
        std::cerr << "   The agent CANNOT continue without a deviceId.\n";
        WinUtils::LogToFile("FATAL: Registration failed → deviceId=0");
        return;
    }

    std::cout << "\n[REGISTER SUCCESS] Device registered with ID = " << deviceId << "\n";

    // ... (Start all worker threads) ...
    _focusThread = std::thread(&Worker::FocusLoop, this);
    _screenThread = std::thread(&Worker::ScreenTimeLoop, this);
    _pollThread = std::thread(&Worker::PollCommandsLoop, this);
    _enforceThread = std::thread(&Worker::EnforceLoop, this);
    _liveThread = std::thread(&Worker::LiveStatusLoop, this);
    
    std::cout << "[INFO] Attempting to start Location Receiver (5558) thread...\n";
    WinUtils::LogToFile("[INFO] Attempting to start Location Receiver (5558) thread...");
    _locationThread = std::thread(&Worker::LocationReceiverLoop, this); 
    
    _zmqThread = std::thread(&Worker::ZmqLoop, this);

    std::cout << "[WORKER] All threads started.\n";
    WinUtils::LogToFile("Worker started successfully");
}
// In Worker.cpp

// NOTE: You must include <zmq.hpp> and have HttpClient::HttpPost available.
// NOTE: Make sure to define these constants outside of the function or as class members.
const std::string BACKEND_LOCATION_PATH = "https://zyberhero.com/api/location";
// const std::string BACKEND_LOCATION_PATH = "http://localhost:8060/api/location";
const std::string LOCATION_PULL_ENDPOINT = "tcp://*:5558"; // Binding address for PULL

void Worker::LocationReceiverLoop()
{
    std::cout << "[LOC_5558] LocationReceiverLoop started.\n";
    WinUtils::LogToFile("[LOC_5558] LocationReceiverLoop started.");

    zmq::context_t ctx;
    zmq::socket_t receiver(ctx, zmq::socket_type::pull); 

    try
    {
        std::cout << "[LOC_5558] Attempting to BIND PULL socket to " << LOCATION_PULL_ENDPOINT << "\n";
        receiver.bind(LOCATION_PULL_ENDPOINT); 
        std::cout << "[LOC_5558] BIND successful.\n";
        
        receiver.set(zmq::sockopt::rcvtimeo, 1000); 
    }
    catch (const zmq::error_t& e)
    {
        // FATAL: The PULL socket could not be opened or bound (e.g., port conflict)
        std::cerr << "[LOC_5558 FATAL ZMQ ERROR] BIND failed. Error code (" << e.num() << "): " << e.what() << "\n";
        WinUtils::LogToFile("[LOC_5558 FATAL ZMQ ERROR] BIND failed: " + std::string(e.what()));
        return; // Exit the thread if ZMQ setup fails
    }
    catch (const std::exception& e) {
        std::cerr << "[LOC_5558 FATAL INIT ERROR] " << e.what() << "\n";
        WinUtils::LogToFile("[LOC_5558 FATAL INIT ERROR] " + std::string(e.what()));
        return;
    }


    while (_running) {
        try {
            zmq::message_t location_msg;

            // PULL socket receives the single JSON frame
            auto res = receiver.recv(location_msg, zmq::recv_flags::none);
            
            if (!res) {
                // This is a normal timeout, no need to log unless debugging flow
                // std::cout << "[LOC_5558] Timeout, checking running flag...\n"; 
                continue; 
            }

            std::string json_str(static_cast<char*>(location_msg.data()), location_msg.size());
            
            std::cout << "[LOC_5558 DATA] Received " << res.value() << " bytes. Posting...\n";

            // --- HTTP POST Attempt ---
            std::string response;
            try {
                // Assuming HttpClient::HttpPost returns the response body on success
                response = g_httpClient.HttpPost(
                    BACKEND_LOCATION_PATH,
                    json_str
                );
                
                // You should check the status code if your HttpClient::HttpPost provides it
                std::cout << "[LOC_5558 HTTP OK] Location posted. Response size: " << response.length() << "\n";

            } catch (const std::exception& e) {
                // LOG: Failure to connect to the backend/send HTTP request
                std::cerr << "[LOC_5558 HTTP ERROR] Post failed: " << e.what() << ". Payload: " << json_str.substr(0, 50) << "...\n";
                WinUtils::LogToFile("LOC_5558 HTTP Post failed: " + std::string(e.what()));
            }

        }
        catch (const zmq::error_t& e) {
            if (e.num() == ETIMEDOUT || e.num() == EAGAIN) continue;
            // LOG: Any non-timeout ZMQ error during receive
            std::cerr << "[LOC_5558 ZMQ RX ERROR] Error code (" << e.num() << "): " << e.what() << "\n";
            WinUtils::LogToFile("LOC_5558 ZMQ RX ERROR: " + std::string(e.what()));
        }
        catch (...) {
            std::cerr << "[LOC_5558 GENERIC ERROR] Unknown exception during loop.\n";
            WinUtils::LogToFile("LOC_5558 GENERIC ERROR: Unknown exception.");
        }
    }

    std::cout << "[LOC_5558] LocationReceiverLoop stopped.\n";
    WinUtils::LogToFile("[LOC_5558] LocationReceiverLoop stopped.");
}

void Worker::ZmqLoop()
{
    std::cout << "[Worker] ZmqLoop started.\n";
    WinUtils::LogToFile("ZmqLoop started");

    zmq::context_t ctx;
    zmq::socket_t subscriber(ctx, zmq::socket_type::sub);

    try
    {
        std::cout << "[ZMQ] Connecting to Alert Source A (5556)...\n";
        subscriber.connect("tcp://127.0.0.1:5556");

        std::cout << "[ZMQ] Connecting to Alert Source B (5557)...\n";
        subscriber.connect("tcp://127.0.0.1:5557");

        std::cout << "[ZMQ] Connecting to Rust Profanity Service (5559)...\n";
        subscriber.connect("tcp://127.0.0.1:5559");

        subscriber.set(zmq::sockopt::subscribe, "ALERT");
        subscriber.set(zmq::sockopt::rcvtimeo, 1000);
    }
    catch (const std::exception& e)
    {
        std::cerr << "[ZMQ ERROR] Init failed: " << e.what() << "\n";
        WinUtils::LogToFile(std::string("ZMQ Init ERROR: ") + e.what());
        return;
    }

    std::cout << "[ZMQ] Listening for alerts from 5556, 5557 & 5559 (Rust profanity service).\n";

    while (_running)
    {
        std::string json_str;
        try
        {
            zmq::message_t msg1;
            auto res = subscriber.recv(msg1, zmq::recv_flags::none);
            if (!res) continue;
            
            // Check if there are more frames (Multipart: Token + JSON)
            if (subscriber.get(zmq::sockopt::rcvmore))
            {
                zmq::message_t msg2;
                res = subscriber.recv(msg2, zmq::recv_flags::none);
                if (!res) continue;
                
                json_str = std::string(static_cast<char*>(msg2.data()), msg2.size());
            }
            else
            {
                // Single frame: "ALERT {"..."}" or just "{"..."}"
                std::string s1(static_cast<char*>(msg1.data()), msg1.size());
                
                // Robust stripping: Find the first '{' to start JSON parsing
                size_t bracePos = s1.find('{');
                if (bracePos != std::string::npos)
                {
                    json_str = s1.substr(bracePos);
                }
                else
                {
                    // Fallback or log error
                    std::cerr << "[ZMQ ERROR] Received message without JSON brace: " << s1 << "\n";
                    continue;
                }
            }

            auto json_data = nlohmann::json::parse(json_str);
            if (!json_data.is_object())
            {
                std::cerr << "[ZMQ ERROR] Received non-object JSON: " << json_str << "\n";
                WinUtils::LogToFile("ZMQ ERROR: non-object JSON: " + json_str);
                continue;
            }

            std::string type = SafeGetStr(json_data, "type");
            std::string url  = SafeGetStr(json_data, "url");
            
            // Extract sender - check if it exists and is not null
            std::string sender = "unknown";
            if (json_data.contains("sender") && !json_data["sender"].is_null() && json_data["sender"].is_string()) {
                sender = json_data["sender"].get<std::string>();
            }

            std::cout << "[ZMQ] Received Alert: " << type << " from " << url << " (Sender: " << sender << ")\n";
            WinUtils::LogToFile("ZMQ Alert: " + type + " from " + url);

            // -----------------------------
            // Build backend alert payload
            // -----------------------------
            json alertPayload;
            alertPayload["timestamp"] = WinUtils::UtcNowISO();   // ✅ Instant-safe
            alertPayload["type"] = type;
            alertPayload["url"]  = url;
            
            // Only add sender if it's not "unknown"
            if (sender != "unknown") {
                alertPayload["sender"] = sender;
            }

            if (!_currentProcessName.empty())
            {
                alertPayload["appName"] = _currentProcessName;
                alertPayload["windowTitle"] = _currentWindowTitle;
            }

            // -----------------------------
            // TEXT_BLOCKED handling
            // -----------------------------
            if (type == "TEXT_BLOCKED")
            {
                // DEBUG: Log the full JSON to understand structure
                std::cout << "[ZMQ DEBUG] Full TEXT_BLOCKED JSON: " << json_data.dump() << "\n";
                
                std::string words;
                
                // Check for bad words in multiple possible locations
                // Priority: badWords > data > words_detected
                
                if (json_data.contains("badWords") && !json_data["badWords"].is_null())
                {
                    std::cout << "[ZMQ DEBUG] Found 'badWords' field\n";
                    auto& bw = json_data["badWords"];
                    if (bw.is_array()) {
                        std::cout << "[ZMQ DEBUG] 'badWords' is array with " << bw.size() << " items\n";
                        for (const auto& w : bw) {
                            if (w.is_string()) {
                                if (!words.empty()) words += ",";
                                words += w.get<std::string>();
                            }
                        }
                    } else if (bw.is_string()) {
                        std::cout << "[ZMQ DEBUG] 'badWords' is string\n";
                        words = bw.get<std::string>();
                    }
                }
                else if (json_data.contains("data") && !json_data["data"].is_null())
                {
                    std::cout << "[ZMQ DEBUG] Found 'data' field\n";
                   auto& d = json_data["data"];
                   if (d.is_array()) {
                       std::cout << "[ZMQ DEBUG] 'data' is array with " << d.size() << " items\n";
                       for (const auto& w : d) {
                           if (w.is_string()) {
                               if (!words.empty()) words += ",";
                               words += w.get<std::string>();
                           }
                       }
                   } else if (d.is_string()) {
                       std::cout << "[ZMQ DEBUG] 'data' is string: " << d.get<std::string>() << "\n";
                       words = d.get<std::string>();
                   } else {
                       std::cout << "[ZMQ DEBUG] 'data' is neither array nor string, type: " << d.type_name() << "\n";
                   }
                }
                else
                {
                    std::cout << "[ZMQ DEBUG] No 'badWords' or 'data' field found. Available keys: ";
                    for (auto& el : json_data.items()) {
                        std::cout << el.key() << " ";
                    }
                    std::cout << "\n";
                }

                if (!words.empty())
                {
                    alertPayload["badWords"] = words; 
                }
                
                // Only add reason if it exists and is not null
                if (json_data.contains("reason") && !json_data["reason"].is_null()) {
                     alertPayload["reason"] = json_data["reason"];
                }

                std::cout << "[ZMQ] Bad words: " << words << "\n";
                WinUtils::LogToFile("ZMQ Bad words: " + words);
            }
            // -----------------------------
            // AUDIO_BLOCKED (from Rust profanity service)
            // -----------------------------
            else if (type == "AUDIO_BLOCKED")
            {
                std::string badWords;
                if (json_data.contains("badWords") && !json_data["badWords"].is_null())
                {
                    auto& bw = json_data["badWords"];
                    if (bw.is_string()) {
                        badWords = bw.get<std::string>();
                    } else if (bw.is_array()) {
                        for (const auto& w : bw) {
                            if (w.is_string()) {
                                if (!badWords.empty()) badWords += ",";
                                badWords += w.get<std::string>();
                            }
                        }
                    }
                }
                alertPayload["badWords"] = badWords;
                
                if (json_data.contains("reason") && !json_data["reason"].is_null()) {
                    alertPayload["reason"] = json_data["reason"];
                }
                if (json_data.contains("strike") && !json_data["strike"].is_null()) {
                    alertPayload["strike"] = json_data["strike"];
                }
                if (json_data.contains("max_strikes") && !json_data["max_strikes"].is_null()) {
                    alertPayload["maxStrikes"] = json_data["max_strikes"];
                }
                
                std::cout << "[ZMQ] AUDIO_BLOCKED: " << badWords << "\n";
                WinUtils::LogToFile("ZMQ AUDIO_BLOCKED: " + badWords);
            }
            // -----------------------------
            // VIDEO_BLOCKED (from Rust NSFW service)
            // -----------------------------
            else if (type == "VIDEO_BLOCKED")
            {
                if (json_data.contains("className") && !json_data["className"].is_null()) {
                    alertPayload["className"] = json_data["className"];
                }
                if (json_data.contains("category") && !json_data["category"].is_null()) {
                    alertPayload["category"] = json_data["category"];
                }
                if (json_data.contains("confidence") && !json_data["confidence"].is_null()) {
                    alertPayload["confidence"] = json_data["confidence"];
                }
                if (json_data.contains("reason") && !json_data["reason"].is_null()) {
                    alertPayload["reason"] = json_data["reason"];
                }
                if (json_data.contains("strike") && !json_data["strike"].is_null()) {
                    alertPayload["strike"] = json_data["strike"];
                }
                if (json_data.contains("max_strikes") && !json_data["max_strikes"].is_null()) {
                    alertPayload["maxStrikes"] = json_data["max_strikes"];
                }
                
                std::string className = SafeGetStr(json_data, "className");
                std::cout << "[ZMQ] VIDEO_BLOCKED: " << className << "\n";
                WinUtils::LogToFile("ZMQ VIDEO_BLOCKED: " + className);
            }
            // -----------------------------
            // IMAGE / API alerts
            // -----------------------------
            else if (type == "IMAGE_BLOCKED" || type == "API_USAGE_EXCEEDED")
            {
                if (json_data.contains("data") && !json_data["data"].is_null())
                {
                    alertPayload["data"] = json_data["data"];
                }
                if (json_data.contains("reason") && !json_data["reason"].is_null())
                {
                    alertPayload["reason"] = json_data["reason"];
                }
            }

            // -----------------------------
            // Send to backend
            // -----------------------------
            SendAlert(alertPayload.dump());
        }
        catch (const json::parse_error& e)
        {
            std::cerr << "[ZMQ PARSE ERROR] " << e.what() << " | Raw string: " << json_str << "\n";
            WinUtils::LogToFile("ZMQ PARSE ERROR: " + std::string(e.what()) + " | Raw: " + json_str);
        }
        catch (const json::type_error& e)
        {
            std::cerr << "[ZMQ TYPE ERROR] " << e.what() << "\n";
            WinUtils::LogToFile("ZMQ TYPE ERROR: " + std::string(e.what()));
        }
        catch (const zmq::error_t& e)
        {
            if (e.num() == ETIMEDOUT || e.num() == EAGAIN)
                continue;

            std::cerr << "[ZMQ ERROR] " << e.what() << "\n";
            WinUtils::LogToFile(std::string("ZMQ ERROR: ") + e.what());
            std::this_thread::sleep_for(std::chrono::seconds(1));
        }
        catch (const std::exception& e)
        {
            std::cerr << "[ZMQ ERROR] Exception: " << e.what() << "\n";
            WinUtils::LogToFile(std::string("ZMQ Exception: ") + e.what());
        }
        catch (...)
        {
            std::cerr << "[ZMQ ERROR] Unknown exception\n";
            WinUtils::LogToFile("ZMQ Unknown exception");
        }
    }

    std::cout << "[Worker] ZmqLoop stopped.\n";
    WinUtils::LogToFile("ZmqLoop stopped");
}

void Worker::FocusLoop()
{
    std::cout << "[Worker] FocusLoop started.\n";

    _currentProcessName.clear();
    _currentWindowTitle.clear();
    _sessionStart = std::chrono::steady_clock::now();

    while (_running)
    {
        try
        {
            auto info = WinUtils::GetForegroundWindowInfo();

            // --- USAGE TRACKING & LIMITS ---
            if (!info.processName.empty())
            {
                CheckDailyReset();

                std::string trackedApp = ToLower(info.processName);
                if (trackedApp.size() >= 4 && trackedApp.substr(trackedApp.size()-4) == ".exe")
                    trackedApp = trackedApp.substr(0, trackedApp.size()-4);

                std::string webApp = DetectWebApp(info.title);
                if (!webApp.empty()) trackedApp = webApp;

                {
                    std::lock_guard<std::mutex> uLock(_usageLock);
                    _appUsageSeconds[trackedApp]++;
                    int used = _appUsageSeconds[trackedApp];

                    if (_appLimitsSeconds.count(trackedApp))
                    {
                        int limit = _appLimitsSeconds[trackedApp];
                        
                        // Check if parent has overridden the limit for today (via relaunch)
                        bool isOverridden = false;
                        {
                            // Already in lock_guard<mutex> uLock(_usageLock) here from parent scope
                            if (_limitOverriddenToday.count(trackedApp)) isOverridden = true;
                        }

                        if (limit > 0 && used > limit && !isOverridden)
                        {
                            // Check suppression window before killing
                            bool suppressed = false;
                            {
                                std::lock_guard<std::mutex> sLock(_suppressLock);
                                auto itS = _suppressKillUntil.find(trackedApp);
                                if (itS != _suppressKillUntil.end() &&
                                    std::chrono::steady_clock::now() < itS->second)
                                {
                                    suppressed = true;
                                }
                            }

                            if (suppressed) {
                                if (used % 5 == 0) {
                                    std::cout << "[USAGE DEBUG] " << trackedApp << " is over limit (" << used << "/" << limit << ") but relaunch suppression is active.\n";
                                }
                            } else {
                                if (used == limit + 1) // Log only on first violation second
                                {
                                    std::cout << "[LIMIT] Time limit reached for " << trackedApp << " (" << limit << "s). Blocking.\n";
                                    WinUtils::LogToFile("[LIMIT] Time limit reached for " + trackedApp);
                                }
                                
                                std::cout << "[ENFORCE] App '" << trackedApp << "' used " << used << "s, limit is " << limit << "s. Killing...\n";

                                // Ensure it's blocked for future enforcement loops
                                {
                                    std::lock_guard<std::mutex> bLock(_blockLock);
                                    _blockedApps.insert(trackedApp);
                                }
                                
                                // Kill immediately
                                KillApp(trackedApp);
                            }
                        }
                        else if (used % 30 == 0) { // Log every 30 seconds for debug
                             std::cout << "[USAGE DEBUG] App: " << trackedApp << " | Used: " << used << "s | Limit: " << limit << "s\n";
                        }
                    }
                    else if (used % 60 == 0) {
                         std::cout << "[USAGE DEBUG] App: " << trackedApp << " | Used: " << used << "s | No Limit Set\n";
                    }
                }
            }
            // --------------------------------

            if (!info.processName.empty())
            {
                // Check if focus changed
                if (info.processName != _currentProcessName ||
                    info.title != _currentWindowTitle)
                {
                    EndCurrentSession();
                    SaveUsageStats();

                    _currentProcessName = info.processName;
                    _currentWindowTitle = info.title;
                    _sessionStart = std::chrono::steady_clock::now();

                    std::cout << "[FOCUSED] New session: "
                              << _currentProcessName
                              << " — " << _currentWindowTitle << "\n";
                }
                else 
                {
                    // Focus is the same. Check duration.
                    // If session is longer than 60 seconds, flush it to DB to keep "Total Active" updated in near real-time.
                    auto now = std::chrono::steady_clock::now();
                    auto duration = std::chrono::duration_cast<std::chrono::seconds>(now - _sessionStart).count();
                    
                    if (duration >= 60) {
                        // Log the chunk
                        EndCurrentSession();
                        SaveUsageStats();
                        
                        // Start new session immediately for the same app
                        _sessionStart = std::chrono::steady_clock::now();
                    }
                }
            }
        }
        catch (const std::exception &e)
        {
            std::cerr << "[ERROR] FocusLoop iteration failed: " << e.what() << "\n";
            WinUtils::LogToFile(std::string("FocusLoop ERROR: ") + e.what());
        }
        catch (...)
        {
            std::cerr << "[ERROR] FocusLoop iteration failed with unknown exception\n";
            WinUtils::LogToFile("FocusLoop ERROR: Unknown exception");
        }

        if (std::chrono::duration_cast<std::chrono::seconds>(std::chrono::steady_clock::now() - _lastSyncTime).count() >= 60)
        {
            SyncDailyUsage();
            _lastSyncTime = std::chrono::steady_clock::now();
        }

        std::this_thread::sleep_for(std::chrono::seconds(1));
    }

    try
    {
        EndCurrentSession();
    }
    catch (const std::exception &e)
    {
        std::cerr << "[ERROR] FocusLoop final EndCurrentSession failed: " << e.what() << "\n";
        WinUtils::LogToFile(std::string("FocusLoop EndCurrentSession ERROR: ") + e.what());
    }
    catch (...)
    {
        std::cerr << "[ERROR] FocusLoop final EndCurrentSession failed\n";
        WinUtils::LogToFile("FocusLoop EndCurrentSession ERROR: Unknown");
    }
    std::cout << "[Worker] FocusLoop stopped. _running=" << (_running ? "true" : "false") << "\n";
    WinUtils::LogToFile("FocusLoop stopped");
}

// ======================================================
// Stop Threads
// ======================================================
void Worker::Stop()
{
    std::cout << "[Worker] Stop() called. _running=" << (_running ? "true" : "false") << "\n";
    
    if (!_running)
    {
        std::cout << "[Worker] Stop() called but already stopped, returning.\n";
        return;
    }
    
    std::cout << "[Worker] Setting _running=false to stop all threads...\n";
    _running = false;

    std::cout << "[Worker] Waiting for focusThread to join...\n";
    if (_focusThread.joinable())
        _focusThread.join();
    
    std::cout << "[Worker] Waiting for screenThread to join...\n";
    if (_screenThread.joinable())
        _screenThread.join();
    
    std::cout << "[Worker] Waiting for pollThread to join...\n";
    if (_pollThread.joinable())
        _pollThread.join();
    
    std::cout << "[Worker] Waiting for enforceThread to join...\n";
    if (_enforceThread.joinable())
        _enforceThread.join();
    
    std::cout << "[Worker] Waiting for liveThread to join...\n";
    if (_liveThread.joinable())
        _liveThread.join();

    std::cout << "[Worker] Waiting for zmqThread to join...\n";
    if (_zmqThread.joinable())
        _zmqThread.join();

    std::cout << "[Worker] All threads joined. Stopped.\n";
}

// ======================================================
// End Focused Session
// ======================================================
void Worker::EndCurrentSession()
{
    if (_currentProcessName.empty())
        return;

    auto now = std::chrono::steady_clock::now();
    int duration = (int)std::chrono::duration_cast<std::chrono::seconds>(now - _sessionStart).count();

    if (duration < 1)
        return;

    // === Generate timestamps ===
    auto nowSys = std::chrono::system_clock::now();
    std::time_t utcRaw = std::chrono::system_clock::to_time_t(nowSys);

    // UTC timestamp (Z format)
    std::stringstream ssUtc;
    ssUtc << std::put_time(std::gmtime(&utcRaw), "%Y-%m-%dT%H:%M:%SZ");

    // Local timestamp
    std::time_t localRaw = utcRaw;
    std::stringstream ssLocal;
    ssLocal << std::put_time(std::localtime(&localRaw), "%Y-%m-%dT%H:%M:%S");

    // Detect if incognito
    bool isIncognito = WinUtils::IsIncognitoWindow(_currentWindowTitle);

    // === Build JSON ===
    json log;
    log["timestamp"] = ssUtc.str();        // UTC time
    log["localTimestamp"] = ssLocal.str(); // Local time
    log["machineName"] = MachineName();
    log["userName"] = getenv("USERNAME") ? getenv("USERNAME") : "";
    log["appName"] = _currentProcessName;
    log["windowTitle"] = _currentWindowTitle;
    log["durationSeconds"] = duration;
    log["executablePath"] = WinUtils::GetExecutablePath(_currentProcessName);
    log["screenTime"] = false;
    log["isIncognito"] = isIncognito;
    log["deviceUuid"] = deviceUuid;
    log["macAddress"] = macAddress;
    log["deviceId"] = deviceId;
    log["childId"] = childId;

    SendActivityLog(log.dump());

    std::cout << "[FOCUSED] Session ended: "
              << _currentProcessName << " (" << duration << "s)"
              << (isIncognito ? " [INCOGNITO]" : "") << "\n";
}

// ======================================================
// Visible Window Tracking
// ======================================================
void Worker::TrackVisibleWindows()
{
    auto windows = WinUtils::GetVisibleWindows();
    std::unordered_set<std::string> nowVisible;
    std::unordered_map<std::string, std::string> nowTitles;

    for (const auto &w : windows)
    {
        std::string procLower = ToLower(w.processName);
        
        // Skip tracking the agent itself
        if (procLower == "childactivitymonitorcpp.exe" || procLower == "childactivitymonitorcpp")
        {
            continue;
        }

        std::string detected = DetectWebApp(w.title);
        std::string baseKey = !detected.empty() ? detected : procLower;

        // Include incognito in key to create separate rows
        std::string key = baseKey + (w.isIncognito ? "#incognito" : "");

        nowVisible.insert(key);
        nowTitles[key] = w.title;
        _visibleProcName[key] = procLower;
    }

    {
        std::lock_guard<std::mutex> g(_visibleAppsLock);

        // ended sessions
        for (auto it = _visibleStartTimes.begin(); it != _visibleStartTimes.end();)
        {
            const std::string &app = it->first;
            if (!nowVisible.count(app))
            {
                // REMOVED: Background duration logging to prevent inflation of "Total Active" time.
                // The dashboard sums up all activity logs. By sending logs for every visible background window,
                // we were multiplying the apparent screen time. 
                // We now only log FOCUSED time (in FocusLoop), which represents actual usage (Digital Wellbeing style).

                /*
                auto start = it->second;
                int duration = (int)std::chrono::duration_cast<std::chrono::seconds>(
                                   std::chrono::steady_clock::now() - start)
                                   .count();

                if (duration >= 2)
                {
                   // ... (omitted) ...
                   SendActivityLog(log.dump());
                }
                */

                it = _visibleStartTimes.erase(it);
                _lastWindowTitles.erase(app);
                _visibleProcName.erase(app);
            }
            else
                ++it;
        }

        // new sessions and title change detection
        for (const auto &kv : nowTitles)
        {
            const std::string &app = kv.first;
            const std::string &title = kv.second;

            if (!_currentlyVisibleApps.count(app))
            {
                _visibleStartTimes[app] = std::chrono::steady_clock::now();
                _lastWindowTitles[app] = title;
            }
            else
            {
                auto lit = _lastWindowTitles.find(app);
                if (lit == _lastWindowTitles.end() || lit->second != title)
                {
                    _lastWindowTitles[app] = title;
                    std::cout << "[LIVE] Title changed for " << app << " -> " << title << "\n";
                    SendLiveStatus();
                }
            }
        }

        _currentlyVisibleApps = nowVisible;
    }
}

// ======================================================
// Screen Time Loop — 5 sec
// ======================================================
void Worker::ScreenTimeLoop()
{
    std::cout << "[Worker] ScreenTimeLoop started.\n";

    while (_running)
    {
        try
        {
            TrackVisibleWindows();
        }
        catch (const std::exception &e)
        {
            std::cerr << "[ERROR] ScreenTimeLoop iteration failed: " << e.what() << "\n";
            WinUtils::LogToFile(std::string("ScreenTimeLoop ERROR: ") + e.what());
        }
        catch (...)
        {
            std::cerr << "[ERROR] ScreenTimeLoop iteration failed with unknown exception\n";
            WinUtils::LogToFile("ScreenTimeLoop ERROR: Unknown exception");
        }
        std::this_thread::sleep_for(std::chrono::seconds(5));
    }

    std::cout << "[Worker] ScreenTimeLoop stopped. _running=" << (_running ? "true" : "false") << "\n";
    WinUtils::LogToFile("ScreenTimeLoop stopped");
}

// ======================================================
// Poll Backend Commands — 5 sec
// ======================================================
// ======================================================
// Poll Backend Commands — 5 sec
// ======================================================
void Worker::PollCommandsLoop()
{
    std::cout << "[Worker] PollCommandsLoop started.\n";

    while (_running)
    {
        try
        {
            // Use deviceUuid as per user's "perfect" logic
            std::string url = "http://localhost:8060/api/commands/pending?deviceUuid=" + deviceUuid;               

            // std::cout << "[POLL] Fetching from: " << url << "\n"; // Hiding noise but keeping url if needed for debug

            std::string res = g_httpClient.HttpGet(url);

            if (!res.empty())
            {
                // std::cout << "[POLL] Raw response: " << res << "\n";

                auto arr = json::parse(res);

                if (arr.is_array() && !arr.empty())
                {
                    std::cout << "[POLL] Found " << arr.size() << " commands\n";

                    // Coalesce commands per app: keep latest, prioritize relaunch > kill > schedule/limit
                    std::unordered_map<std::string, Command> perApp;

                    for (auto &x : arr)
                    {
                        try
                        {
                            Command cmd;
                            cmd.id = SafeGetInt(x, "id");

                            cmd.appName = SafeGetStr(x, "appName");
                            cmd.action = SafeGetStr(x, "action");
                            cmd.schedule = SafeGetStr(x, "schedule");

                            cmd.isActive = x.value("isActive", true);

                            if (!cmd.isActive)
                                continue;

                            std::string appKey = ToLower(cmd.appName);

                            auto it = perApp.find(appKey);
                            if (it == perApp.end())
                            {
                                perApp[appKey] = cmd;
                            }
                            else
                            {
                                // Priority: relaunch > kill > limit-usage/schedule
                                auto priority = [](const std::string &action)
                                {
                                    if (action == "relaunch")
                                        return 3;
                                    if (action == "kill")
                                        return 2;
                                    if (action == "limit-usage" || action == "schedule")
                                        return 1;
                                    return 0;
                                };

                                int newPriority = priority(cmd.action);
                                int existingPriority = priority(it->second.action);

                                if (newPriority > existingPriority ||
                                    (newPriority == existingPriority && cmd.id > it->second.id))
                                {
                                    perApp[appKey] = cmd;
                                }
                            }
                        }
                        catch (const std::exception &e)
                        {
                            std::cerr << "[POLL ERROR] Failed to parse command: " << e.what()
                                      << " | JSON: " << x.dump() << "\n";
                        }
                    }

                    // Process coalesced commands
                    bool anyExecuted = false;
                    for (const auto &kv : perApp)
                    {
                        const Command &cmd = kv.second;

                        // Deduplicate: skip if already applied for this app
                        {
                            std::lock_guard<std::mutex> lock(_cmdLock);
                            auto it = _lastAppliedCmdId.find(kv.first);
                            if (it != _lastAppliedCmdId.end() && it->second == cmd.id)
                            {
                                continue;
                            }
                            _lastAppliedCmdId[kv.first] = cmd.id;
                        }

                        std::cout << "[POLL] Executing coalesced command: id=" << cmd.id
                                  << " action=" << cmd.action
                                  << " app=" << cmd.appName << "\n";

                        HandleCommand(cmd);
                        anyExecuted = true;
                    }
                    
                    if (anyExecuted) {
                        SaveUsageStats();
                    }
                }
            }
        }
        catch (const std::exception &e)
        {
            std::cerr << "[POLL ERROR] Exception in PollCommandsLoop: " << e.what() << "\n";
            WinUtils::LogToFile(std::string("PollCommandsLoop ERROR: ") + e.what());
        }
        catch (...)
        {
            std::cerr << "[POLL ERROR] Unknown exception in PollCommandsLoop\n";
            WinUtils::LogToFile("PollCommandsLoop ERROR: Unknown exception");
        }

        std::this_thread::sleep_for(std::chrono::seconds(5));
    }

    std::cout << "[Worker] PollCommandsLoop stopped.\n";
}

// ======================================================
// Handle Command
// ======================================================
void Worker::HandleCommand(const Command &cmd)
{
    std::string app = ToLower(cmd.appName);
    if (app.length() >= 4 && app.compare(app.length() - 4, 4, ".exe") == 0) {
        app.erase(app.length() - 4);
    }

    std::cout << "[COMMAND] Received: id=" << cmd.id
              << " action=" << cmd.action
              << " app=" << cmd.appName
              << " isActive=" << cmd.isActive << "\n";

    if (cmd.action == "kill")
    {
        std::lock_guard<std::mutex> lock(_blockLock);
        _blockedApps.insert(app);
        
        // Also clear any previous override if parent explicitly kills
        {
            std::lock_guard<std::mutex> uLock(_usageLock);
            _limitOverriddenToday.erase(app);
        }

        std::cout << "[COMMAND] ✔ Kill command registered for: " << app << " (enforcement loop will handle)\n";
    }
    else if (cmd.action == "relaunch")
    {
        {
            std::lock_guard<std::mutex> lock(_blockLock);
            _blockedApps.erase(app);
        }
        {
            std::lock_guard<std::mutex> uLock(_usageLock);
            _limitOverriddenToday.insert(app);
            std::cout << "[COMMAND] Limit Override set for: " << app << " (until next reset or limit change)\n";
        }
        {
            std::lock_guard<std::mutex> lock(_suppressLock);
            _suppressKillUntil[app] = std::chrono::steady_clock::now() + std::chrono::seconds(15);
            std::cout << "[COMMAND] Suppression window active for: " << app << " (15s)\n";
        }
        LaunchApp(app);
        std::cout << "[COMMAND] ✔ Relaunch executed for: " << app << "\n";
    }
    else if (cmd.action == "limit-usage")
    {
        try {
            // Parse the schedule JSON: {"dailyLimitMinutes":60,"resetTime":"00:00","activeDays":"ALL"}
            json scheduleData = json::parse(cmd.schedule);
            
            int dailyLimitMinutes = scheduleData.value("dailyLimitMinutes", 0);
            int seconds = dailyLimitMinutes * 60;

            std::cout << "[COMMAND] Setting daily limit for " << app << " to " << dailyLimitMinutes << " minutes.\n";
            
            {
                std::lock_guard<std::mutex> lock(_usageLock);
                if (seconds <= 0) {
                    _appLimitsSeconds.erase(app);
                } else {
                    _appLimitsSeconds[app] = seconds;
                }
                
                // CRITICAL: Changing the limit clears the override to re-evaluate the new rule
                _limitOverriddenToday.erase(app);
            }
        } catch (const std::exception& e) {
            std::cerr << "[COMMAND ERROR] Failed to parse limit-usage for " << app << ": " << e.what() << "\n";
        }
    }
    else if (cmd.action == "schedule")
    {
        std::cout << "[COMMAND] Schedule command (not yet implemented): " << cmd.schedule << "\n";
    }
    else if (cmd.action == "set_limit")
    {
        try {
            int seconds = std::stoi(cmd.schedule);
            std::cout << "[COMMAND] Setting usage limit for " << app << " to " << seconds << " seconds.\n";
            
            {
                std::lock_guard<std::mutex> lock(_usageLock);
                // If limit is 0, we can remove it or set to 0 (meaning no limit in our logic if check is > 0)
                if (seconds <= 0) _appLimitsSeconds.erase(app);
                else _appLimitsSeconds[app] = seconds;
            }
            SaveUsageStats();
        } catch (...) {
            std::cerr << "[COMMAND ERROR] Invalid limit value for " << app << ": " << cmd.schedule << "\n";
        }
    }
}
// ======================================================
// Enforcement Loop — kills blocked apps repeatedly
// ======================================================
void Worker::EnforceLoop()
{
    std::cout << "[Worker] EnforcementLoop started.\n";
    WinUtils::LogToFile("EnforcementLoop started");

    while (_running)
    {
        try
        {
            std::unordered_set<std::string> copy;
            {
                std::lock_guard<std::mutex> lock(_blockLock);
                copy = _blockedApps;
            }

            for (auto &app : copy)
            {
                // Check suppression window
                {
                    std::lock_guard<std::mutex> lock(_suppressLock);
                    auto it = _suppressKillUntil.find(app);
                    if (it != _suppressKillUntil.end() &&
                        std::chrono::steady_clock::now() < it->second)
                    {
                        std::cout << "[ENFORCE] Skipping " << app << " (suppression active)\n";
                        continue;
                    }
                }

                KillApp(app);
            }
        }
        catch (const std::exception &e)
        {
            std::cerr << "[ERROR] EnforceLoop iteration failed: " << e.what() << "\n";
            WinUtils::LogToFile(std::string("EnforceLoop ERROR: ") + e.what());
        }
        catch (...)
        {
            std::cerr << "[ERROR] EnforceLoop iteration failed with unknown exception\n";
            WinUtils::LogToFile("EnforceLoop ERROR: Unknown exception");
        }

        std::this_thread::sleep_for(std::chrono::seconds(3));
    }

    std::cout << "[Worker] EnforcementLoop stopped. _running=" << (_running ? "true" : "false") << "\n";
    WinUtils::LogToFile("EnforcementLoop stopped");
}

// ======================================================
// Kill App (Native + Browser Tabs)
// ======================================================
void Worker::KillApp(const std::string &appName)
{
    bool killed = false;
    std::string appLower = ToLower(appName);

    // Safety: Never kill the agent itself
    if (appLower == "childactivitymonitorcpp" || appLower == "childactivitymonitorcpp.exe")
    {
        std::cout << "[KILL] Safety check: Cannot kill the agent itself\n";
        return;
    }

    // web app?
    if (WebAppPatterns.count(appLower))
    {
        auto keywords = WebAppPatterns.at(appLower);
        auto windows = WinUtils::FindBrowserWindowsByKeywords(keywords);

        if (!windows.empty())
        {
            std::cout << "[KILL] Found " << windows.size() << " matching window(s) for: " << appName << "\n";

            for (HWND h : windows)
            {
                // Try Ctrl+W first
                WinUtils::CloseChromeTab(h);
                std::this_thread::sleep_for(std::chrono::milliseconds(300));

                // Check if window still exists, fallback to WM_CLOSE
                if (IsWindow(h) && IsWindowVisible(h))
                {
                    std::cout << "[KILL] Ctrl+W failed, using WM_CLOSE for window\n";
                    PostMessage(h, WM_CLOSE, 0, 0);
                }

                killed = true;
            }
            std::cout << "[KILL] Closed tab(s) for web app: " << appName << "\n";
        }
    }
    else
    {
        // native process termination (existing approach)
        auto snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if (snap != INVALID_HANDLE_VALUE)
        {
            PROCESSENTRY32 pe{};
            pe.dwSize = sizeof(pe);
            if (Process32First(snap, &pe))
            {
                do
                {
                    std::string proc = ToLower(WinUtils::WStringToString(pe.szExeFile));
                    if (proc == appLower + ".exe" || proc == appLower)
                    {
                        HANDLE h = OpenProcess(PROCESS_TERMINATE, FALSE, pe.th32ProcessID);
                        if (h)
                        {
                            TerminateProcess(h, 1);
                            CloseHandle(h);
                            killed = true;
                            std::cout << "[KILL] Terminated native process: " << proc << "\n";
                            WinUtils::LogToFile("KILL: Terminated native process: " + proc);
                        }
                    }
                } while (Process32Next(snap, &pe));
            }
            CloseHandle(snap);
        }
    }

    if (!killed) {
        std::cout << "[KILL] App not found: " << appName << "\n";
        WinUtils::LogToFile("KILL: App not found or could not be killed: " + appName);
    }
}

// ======================================================
// Launch App
// ======================================================
void Worker::LaunchApp(const std::string &appName)
{
    std::string a = ToLower(appName);
    static const std::unordered_map<std::string, std::string> urls = {
        {"google-chat", "https://chat.google.com"},
        {"whatsapp", "https://web.whatsapp.com"},
        {"youtube", "https://youtube.com"},
        {"gmail", "https://mail.google.com"},
        {"postman", "https://web.postman.co"}};

    if (urls.count(a))
    {
        ShellExecuteA(NULL, "open", urls.at(a).c_str(), NULL, NULL, SW_SHOWNORMAL);
        std::cout << "[LAUNCH] Opened URL for " << appName << "\n";
        return;
    }

    std::string exe = appName + ".exe";
    ShellExecuteA(NULL, "open", exe.c_str(), NULL, NULL, SW_SHOWNORMAL);
    std::cout << "[LAUNCH] Launched native app: " << exe << "\n";
}

// ======================================================
// Send Activity Logs (Unified normalization as per user logic)
// ======================================================
void Worker::SendActivityLog(const std::string &body)
{
    try
    {
        // Parse the incoming JSON body
        json payload = json::parse(body);

        std::string rawApp = payload.value("appName", "");
        std::string windowTitle = payload.value("windowTitle", "");
        std::string normApp;

        // 1) Detect web-app from window title
        std::string detected = DetectWebApp(windowTitle);
        if (!detected.empty())
        {
            normApp = detected;   // youtube, whatsapp, google-chat, gmail, postman
        }
        else
        {
            // 2) Normalize native process name
            normApp = ToLower(rawApp);

            // Remove ".exe"
            if (normApp.size() > 4 && 
                normApp.substr(normApp.size() - 4) == ".exe")
            {
                normApp = normApp.substr(0, normApp.size() - 4);
            }
        }

        // Store final cleaned name
        payload["appName"] = normApp;

        // Always attach device identity (including deviceUuid as per user logic)
        payload["deviceUuid"] = deviceUuid;
        payload["macAddress"] = macAddress;
        payload["deviceId"] = deviceId;
        payload["childId"] = childId;

        std::string finalBody = payload.dump();

        std::cout << "[ACTIVITY] Sending: " << finalBody << "\n";
        WinUtils::LogToFile("ACTIVITY POST: " + finalBody);

        // Send final cleaned payload
        g_httpClient.HttpPost("https://zyberhero.com/api/activity", finalBody);
    }
    catch (const std::exception &e)
    {
        std::cerr << "[ERROR] SendActivityLog failed: " << e.what() << "\n";
        WinUtils::LogToFile(std::string("SendActivityLog ERROR: ") + e.what());
    }
    catch (...)
    {
        std::cerr << "[ERROR] SendActivityLog failed with unknown exception\n";
        WinUtils::LogToFile("SendActivityLog ERROR: Unknown exception");
    }
}



// ======================================================
// Get Current Timestamp (ISO 8601)
// ======================================================
std::string Worker::GetCurrentTimestamp()
{
    auto nowSys = std::chrono::system_clock::now();
    std::time_t utcRaw = std::chrono::system_clock::to_time_t(nowSys);
    std::stringstream ssUtc;
    ssUtc << std::put_time(std::gmtime(&utcRaw), "%Y-%m-%dT%H:%M:%SZ");
    return ssUtc.str();
}

// ======================================================
// Send Alert to Backend
// ======================================================
void Worker::SendAlert(const std::string &body)
{
    try
    {
        json alertPayload = json::parse(body);

        // ✅ Validate device identity before sending
        if (deviceUuid.empty() && deviceId == 0)
        {
            std::cerr << "[ALERT] Missing deviceUuid/deviceId. Alert not sent.\n";
            return;
        }

        // ✅ Attach ONLY fields expected by backend
        alertPayload["deviceUuid"] = deviceUuid;
        alertPayload["deviceId"]   = deviceId;

        // ✅ Ensure timestamp exists (Instant-compatible)
        if (!alertPayload.contains("timestamp"))
        {
            alertPayload["timestamp"] = WinUtils::UtcNowISO(); // must end with Z
        }

        std::string finalBody = alertPayload.dump();

        std::cout << "[ALERT] Sending alert to backend: " << finalBody << "\n";
        WinUtils::LogToFile("Alert POST: " + finalBody);

        std::string response = g_httpClient.HttpPost(
            // "https://zyberhero.com/api/alerts",
            "http://localhost:8060/api/alerts",
            finalBody
        );
        
        // Log the response from backend
        if (!response.empty())
        {
            std::cout << "[ALERT] Backend response: " << response << "\n";
            WinUtils::LogToFile("Alert POST Response: " + response);
        }
        else
        {
            std::cout << "[ALERT] Backend returned empty response (may indicate success)\n";
        }
    }
    catch (const std::exception &e)
    {
        std::cerr << "[ERROR] SendAlert failed: " << e.what() << "\n";
        WinUtils::LogToFile(std::string("SendAlert ERROR: ") + e.what());
    }
    catch (...)
    {
        std::cerr << "[ERROR] SendAlert failed with unknown exception\n";
        WinUtils::LogToFile("SendAlert ERROR: Unknown exception");
    }
}

// ======================================================
// Live Status — list of visible windows
// ======================================================
void Worker::SendLiveStatus()
{
    auto windows = WinUtils::GetVisibleWindows();

    json apps = json::array();
    for (const auto &w : windows)
    {
        std::string procLower = ToLower(w.processName);
        
        // Skip the agent itself
        if (procLower == "childactivitymonitorcpp.exe" || procLower == "childactivitymonitorcpp")
        {
            continue;
        }

        // detect web app first; otherwise use process name (lower)
        std::string detected = DetectWebApp(w.title);
        std::string appName = !detected.empty() ? detected : procLower;

        // ✅ Only include appName and windowTitle (per backend LiveAppDto)
        json entry;
        entry["appName"] = appName;
        entry["windowTitle"] = w.title;
        apps.push_back(entry);

        std::cout << "[LIVE STATUS] " << appName << " | " << w.title << "\n";
    }

    // ✅ MATCH EXACT BACKEND PAYLOAD STRUCTURE: LiveStatusRequestDto
    json payload = json::object();
    payload["deviceUuid"] = deviceUuid;
    payload["deviceId"] = deviceId;  // Long type
    payload["machineName"] = MachineName();
    payload["apps"] = apps;

    std::string url = "https://zyberhero.com/api/live-status";
    // std::string url = "http://localhost:8060/api/live-status";

    std::cout << "[LIVE STATUS DEBUG] URL: " << url << "\n";
    std::cout << "[LIVE STATUS DEBUG] Payload: " << payload.dump() << "\n";

    try
    {
        auto res = g_httpClient.HttpPost(url, payload.dump());
        
        if (res.empty())
        {
            std::cerr << "[LIVE STATUS] failed to post live-status\n";
        }
        else
        {
            std::cout << "[LIVE STATUS] Posted successfully. Response: " << res << "\n";
        }
    }
    catch (const std::exception &e)
    {
        std::cerr << "[ERROR] SendLiveStatus failed: " << e.what() << "\n";
    }
    catch (...)
    {
        std::cerr << "[ERROR] SendLiveStatus failed with unknown exception\n";
    }
}
// ======================================================
// LiveStatus Loop — 2 sec
// ======================================================
void Worker::LiveStatusLoop()
{
    std::cout << "[Worker] LiveStatusLoop started.\n";
    WinUtils::LogToFile("LiveStatusLoop started");

    while (_running)
    {
        try
        {
            SendLiveStatus();
        }
        catch (const std::exception &e)
        {
            std::cerr << "[ERROR] LiveStatusLoop iteration failed: " << e.what() << "\n";
            WinUtils::LogToFile(std::string("LiveStatusLoop ERROR: ") + e.what());
        }
        catch (...)
        {
            std::cerr << "[ERROR] LiveStatusLoop iteration failed with unknown exception\n";
            WinUtils::LogToFile("LiveStatusLoop ERROR: Unknown exception");
        }
        std::this_thread::sleep_for(std::chrono::seconds(2));
    }

    std::cout << "[Worker] LiveStatusLoop stopped. _running=" << (_running ? "true" : "false") << "\n";
    WinUtils::LogToFile("LiveStatusLoop stopped");
}

// ======================================================
// App Usage Limits & Persistence
// ======================================================
const std::string USAGE_STATS_PATH = "C:\\ProgramData\\ParentalMonitor\\usage_stats.json";

void Worker::CheckDailyReset()
{
    // Get current date string YYYY-MM-DD
    auto now = std::chrono::system_clock::now();
    std::time_t t = std::chrono::system_clock::to_time_t(now);
    std::tm local{};
    localtime_s(&local, &t);

    std::ostringstream oss;
    oss << std::put_time(&local, "%Y-%m-%d");
    std::string today = oss.str();

    bool changed = false;
    {
        std::lock_guard<std::mutex> lock(_usageLock);
        if (_lastUsageDate != today)
        {
            std::cout << "[USAGE] New day detected (" << today << "). Resetting usage stats.\n";
            _lastUsageDate = today;
            _appUsageSeconds.clear();
            
            // Unblock all apps on new day so they can be used until limit is hit again
            _blockedApps.clear(); 
            changed = true;
        }
    }

    if (changed) {
        SaveUsageStats();
    }
}

void Worker::LoadUsageStats()
{
    std::lock_guard<std::mutex> lock(_usageLock);
    _appUsageSeconds.clear();
    _appLimitsSeconds.clear();
    
    // Default last usage to today if file missing
    auto now = std::chrono::system_clock::now();
    std::time_t t = std::chrono::system_clock::to_time_t(now);
    std::tm local{};
    localtime_s(&local, &t);
    std::ostringstream oss;
    oss << std::put_time(&local, "%Y-%m-%d");
    _lastUsageDate = oss.str();

    std::ifstream f(USAGE_STATS_PATH);
    if (!f.good()) return;

    try {
        json j;
        f >> j;

        _lastUsageDate = j.value("date", _lastUsageDate);

        if (j.contains("usage")) {
            for (auto& item : j["usage"].items()) {
                _appUsageSeconds[item.key()] = item.value().get<int>();
            }
        }
        if (j.contains("limits")) {
            for (auto& item : j["limits"].items()) {
                _appLimitsSeconds[item.key()] = item.value().get<int>();
            }
        }
        if (j.contains("overrides") && j["overrides"].is_array()) {
            for (auto& item : j["overrides"]) {
                if (item.is_string()) {
                    _limitOverriddenToday.insert(item.get<std::string>());
                }
            }
        }
        
        {
            std::lock_guard<std::mutex> lockB(_blockLock);
            _blockedApps.clear();
            if (j.contains("blocked") && j["blocked"].is_array()) {
                for (auto& item : j["blocked"]) {
                    if (item.is_string()) {
                        _blockedApps.insert(item.get<std::string>());
                    }
                }
            }
        }

        {
            std::lock_guard<std::mutex> lockC(_cmdLock);
            _lastAppliedCmdId.clear();
            if (j.contains("lastAppliedCmds")) {
                for (auto& item : j["lastAppliedCmds"].items()) {
                    _lastAppliedCmdId[item.key()] = item.value().get<int>();
                }
            }
        }

        std::cout << "[USAGE] Loaded complete state. Apps blocked: " << _blockedApps.size() << "\n";
    } catch (...) {
        std::cerr << "[USAGE] Failed to parse usage stats file.\n";
    }
}

void Worker::SaveUsageStats()
{
    // Lock happens at call site usually, or we lock here? 
    // Ideally we should lock, but we might already hold lock. 
    // Let's assume caller does NOT hold lock for now, or use recursive mutex? 
    // std::mutex is not recursive. 
    // Let's protect internal access but we need to be careful about not calling Save from inside another Lock.
    // CheckDailyReset holds lock -> calls SaveUsageStats. Deadlock! 
    // Fix: Pass 'locked' flag or create a private helper.
    // Simpler: Just do the file I/O. The map copy is fast.
    
    // We will make specific internal helpers if needed, but for now let's avoid deadlock 
    // by NOT locking in SaveUsageStats and ensuring callers lock.
    // OR create a copy under lock and save the copy.
    
    // Correction: LoadCurrentSession updates usage and calls Save.
    // PollCommand calls Save.
    
    // Let's make SaveUsageStats acquire the lock. 
    // And CheckDailyReset will NOT call SaveUsageStats? Or it will call it after unlocking.
    
    // Refactored CheckDailyReset to unlock before save.
    
    json j;
    {
        std::lock_guard<std::mutex> lock(_usageLock);
        j["date"] = _lastUsageDate;
        j["usage"] = _appUsageSeconds;
        j["limits"] = _appLimitsSeconds;
        j["overrides"] = std::vector<std::string>(_limitOverriddenToday.begin(), _limitOverriddenToday.end());
    }

    {
        std::lock_guard<std::mutex> lockB(_blockLock);
        j["blocked"] = std::vector<std::string>(_blockedApps.begin(), _blockedApps.end());
    }

    {
        std::lock_guard<std::mutex> lockC(_cmdLock);
        j["lastAppliedCmds"] = _lastAppliedCmdId;
    }

    try {
        std::ofstream f(USAGE_STATS_PATH);
        f << j.dump(4);
    } catch (...) {
        std::cerr << "[USAGE] Failed to save usage stats.\n";
    }
}

// ======================================================
// WinUtils Time Helpers
// ======================================================
std::string WinUtils::UtcNowISO()
{
    std::time_t now = std::time(nullptr);
    std::tm gmt{};
    gmtime_s(&gmt, &now);

    std::ostringstream oss;
    oss << std::put_time(&gmt, "%Y-%m-%dT%H:%M:%SZ");
    return oss.str();
}

std::string WinUtils::LocalNowISO()
{
    std::time_t now = std::time(nullptr);
    std::tm local{};
    localtime_s(&local, &now);

    std::ostringstream oss;
    oss << std::put_time(&local, "%Y-%m-%dT%H:%M:%S") << "+05:30"; 
    return oss.str();
}
