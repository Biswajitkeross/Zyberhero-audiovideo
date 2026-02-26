/**
 * Profanity Filter Client - C++ Header
 * 
 * Connect to Rust Profanity Filter Service via TCP Socket
 * Works across network - just provide the IP address of the computer running the Rust service
 * 
 * Usage:
 *   1. Run profanity_filter.exe on Computer A
 *   2. In your C++ app on Computer B:
 *      
 *      ProfanityFilterClient client;
 *      if (client.connect("192.168.1.100", 9999)) {
 *          while (true) {
 *              ProfanityEvent event;
 *              if (client.readEvent(event)) {
 *                  // Process event with your AI agent
 *                  if (event.eventType == "BAD_WORD") {
 *                      // Handle profanity detection
 *                  }
 *              }
 *          }
 *      }
 */

#pragma once

#ifdef _WIN32
    #define WIN32_LEAN_AND_MEAN
    #include <winsock2.h>
    #include <ws2tcpip.h>
    #pragma comment(lib, "ws2_32.lib")
#else
    #include <sys/socket.h>
    #include <netinet/in.h>
    #include <arpa/inet.h>
    #include <unistd.h>
    #define SOCKET int
    #define INVALID_SOCKET -1
    #define SOCKET_ERROR -1
    #define closesocket close
#endif

#include <string>
#include <iostream>
#include <sstream>

// Default port for profanity filter service
constexpr int PROFANITY_FILTER_PORT = 5559;

/**
 * Event received from Profanity Filter Service
 */
struct ProfanityEvent {
    std::string timestamp;   // "2026-02-19 10:30:00.123"
    std::string eventType;   // "BAD_WORD", "SERVICE_STARTED", "APPS_CLOSED", etc.
    std::string message;     // Details (for BAD_WORD: "fuck|1|3" means word|strike|maxStrikes)
    
    // Parse BAD_WORD message to get details
    void parseBadWordMessage(std::string& word, int& strike, int& maxStrikes) const {
        if (eventType != "BAD_WORD") return;
        
        std::stringstream ss(message);
        std::getline(ss, word, '|');
        
        std::string strikeStr, maxStr;
        std::getline(ss, strikeStr, '|');
        std::getline(ss, maxStr, '|');
        
        strike = std::stoi(strikeStr);
        maxStrikes = std::stoi(maxStr);
    }
};

/**
 * Client to connect to Profanity Filter Service
 */
class ProfanityFilterClient {
public:
    ProfanityFilterClient() : m_socket(INVALID_SOCKET), m_connected(false) {
#ifdef _WIN32
        WSADATA wsaData;
        WSAStartup(MAKEWORD(2, 2), &wsaData);
#endif
    }
    
    ~ProfanityFilterClient() {
        disconnect();
#ifdef _WIN32
        WSACleanup();
#endif
    }
    
    /**
     * Connect to Profanity Filter Service
     * @param ipAddress IP address of computer running profanity_filter.exe
     * @param port TCP port (default: 9999)
     * @return true if connected successfully
     */
    bool connect(const std::string& ipAddress, int port = PROFANITY_FILTER_PORT) {
        m_socket = socket(AF_INET, SOCK_STREAM, IPPROTO_TCP);
        if (m_socket == INVALID_SOCKET) {
            std::cerr << "Failed to create socket" << std::endl;
            return false;
        }
        
        sockaddr_in serverAddr{};
        serverAddr.sin_family = AF_INET;
        serverAddr.sin_port = htons(port);
        inet_pton(AF_INET, ipAddress.c_str(), &serverAddr.sin_addr);
        
        if (::connect(m_socket, (sockaddr*)&serverAddr, sizeof(serverAddr)) == SOCKET_ERROR) {
            std::cerr << "Failed to connect to " << ipAddress << ":" << port << std::endl;
            closesocket(m_socket);
            m_socket = INVALID_SOCKET;
            return false;
        }
        
        m_connected = true;
        std::cout << "Connected to Profanity Filter at " << ipAddress << ":" << port << std::endl;
        return true;
    }
    
    /**
     * Disconnect from service
     */
    void disconnect() {
        if (m_socket != INVALID_SOCKET) {
            closesocket(m_socket);
            m_socket = INVALID_SOCKET;
        }
        m_connected = false;
    }
    
    /**
     * Check if connected
     */
    bool isConnected() const {
        return m_connected;
    }
    
    /**
     * Read next event from service (blocking)
     * @param event Output event structure
     * @return true if event was read, false if disconnected
     */
    bool readEvent(ProfanityEvent& event) {
        if (!m_connected) return false;
        
        // Read until newline (each event is JSON + newline)
        std::string line;
        char ch;
        
        while (true) {
            int result = recv(m_socket, &ch, 1, 0);
            if (result <= 0) {
                m_connected = false;
                return false;
            }
            
            if (ch == '\n') break;
            line += ch;
        }
        
        // Parse JSON (simple parser - assumes well-formed JSON)
        return parseJson(line, event);
    }
    
    /**
     * Read event with timeout (non-blocking)
     * @param event Output event structure
     * @param timeoutMs Timeout in milliseconds
     * @return true if event was read, false if timeout or disconnected
     */
    bool readEventWithTimeout(ProfanityEvent& event, int timeoutMs) {
        if (!m_connected) return false;
        
        fd_set readSet;
        FD_ZERO(&readSet);
        FD_SET(m_socket, &readSet);
        
        timeval timeout;
        timeout.tv_sec = timeoutMs / 1000;
        timeout.tv_usec = (timeoutMs % 1000) * 1000;
        
        int result = select((int)m_socket + 1, &readSet, nullptr, nullptr, &timeout);
        if (result <= 0) return false;
        
        return readEvent(event);
    }

private:
    SOCKET m_socket;
    bool m_connected;
    
    // Simple JSON parser for our specific format
    bool parseJson(const std::string& json, ProfanityEvent& event) {
        // Expected format: {"timestamp":"...","event_type":"...","message":"..."}
        event.timestamp = extractJsonValue(json, "timestamp");
        event.eventType = extractJsonValue(json, "event_type");
        event.message = extractJsonValue(json, "message");
        return !event.eventType.empty();
    }
    
    std::string extractJsonValue(const std::string& json, const std::string& key) {
        std::string searchKey = "\"" + key + "\":\"";
        size_t start = json.find(searchKey);
        if (start == std::string::npos) return "";
        
        start += searchKey.length();
        size_t end = json.find("\"", start);
        if (end == std::string::npos) return "";
        
        return json.substr(start, end - start);
    }
};


// ============================================================================
// EXAMPLE USAGE
// ============================================================================
/*

#include "profanity_filter_client.h"
#include <thread>

// Callback for your AI agent
void onProfanityDetected(const std::string& word, int strike, int maxStrikes) {
    std::cout << "AI Agent received: " << word << " (" << strike << "/" << maxStrikes << ")" << std::endl;
    
    // Your AI agent processing here...
}

int main() {
    ProfanityFilterClient client;
    
    // Connect to Rust service on another computer
    // Replace with actual IP address of computer running profanity_filter.exe
    if (!client.connect("192.168.1.100", 9999)) {
        std::cerr << "Cannot connect to profanity filter service" << std::endl;
        return 1;
    }
    
    std::cout << "Listening for profanity events..." << std::endl;
    
    while (client.isConnected()) {
        ProfanityEvent event;
        
        if (client.readEvent(event)) {
            std::cout << "[" << event.timestamp << "] " 
                      << event.eventType << ": " << event.message << std::endl;
            
            // Handle specific events
            if (event.eventType == "BAD_WORD") {
                std::string word;
                int strike, maxStrikes;
                event.parseBadWordMessage(word, strike, maxStrikes);
                
                onProfanityDetected(word, strike, maxStrikes);
            }
            else if (event.eventType == "APPS_CLOSED") {
                std::cout << "Media apps were closed due to profanity!" << std::endl;
            }
            else if (event.eventType == "SERVICE_STOPPED") {
                std::cout << "Profanity filter service stopped" << std::endl;
                break;
            }
        }
    }
    
    return 0;
}

*/
