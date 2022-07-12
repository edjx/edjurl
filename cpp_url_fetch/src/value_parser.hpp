#pragma once

#include <cstdint>
#include <string>
#include <optional>

// Value that is stored in the KV store.
struct Value {
    std::string url;
    std::optional<std::string> password;
};

/**
 * @brief Reconstruct Value from the serialized bytes.
 * 
 * Serialized Value looks like this:
 * | URL_LENGTH | URL DATA | PASSWORD_LENGTH | PASSWORD_DATA |
 * 
 * @param result Result Value
 * @param source_bytes Source serialized bytes
 * @return true Successfully parsed
 * @return false Parsing failed
 */
bool deserialize_value(Value & result, const std::vector<uint8_t> & source_bytes);
