#include <cstdint>
#include <string>
#include <optional>
#include <vector>

#include "value_parser.hpp"

std::string to_string(const std::vector<uint8_t> & vec) {
    std::string out;
    out.reserve(vec.size());

    for (uint8_t b : vec) {
        out += b;
    }

    return out;
}

// "bytes" array must be exactly 4 bytes long!
uint32_t from_bigend_bytes_32(const uint8_t * const bytes) {
    return (uint32_t)(bytes[0]) << 24
        | (uint32_t)(bytes[1]) << 16
        | (uint32_t)(bytes[2]) << 8
        | (uint32_t)(bytes[3]);
}

bool deserialize_value(Value & result, const std::vector<uint8_t> & source_bytes) {
    std::string url;
    std::optional<std::string> password;

    enum class Member {
        Url,
        Password,
        None
    };
    enum class Field {
        Size,
        Data
    };

    Member current_member = Member::Url;
    Field current_field = Field::Size;

    uint8_t size_buf[4];
    int size_buf_i = 0;
    uint32_t size;

    std::vector<uint8_t> data;

    for (const uint8_t byte : source_bytes) {
        if (current_member == Member::None) {
            return false;
        }

        switch (current_field) {
            case Field::Size: {
                size_buf[size_buf_i++] = byte;
                if (size_buf_i >= 4) {
                    size = from_bigend_bytes_32(size_buf);
                    size_buf_i = 0;
                    current_field = Field::Data;
                }
                break;
            }
            case Field::Data: {
                if (size <= 0) {
                    // This should not happen
                    return false;
                }
                data.push_back(byte);
                size--;
                break;
            }
            default: {
                return false;
            }
        }

        if (current_field == Field::Data) {
            if (size <= 0) {
                switch (current_member) {
                    case Member::Url:
                        url = to_string(data);
                        current_member = Member::Password;
                        current_field = Field::Size;
                        data.clear();
                        break;
                    case Member::Password:
                        password = to_string(data);
                        current_member = Member::None;
                        current_field = Field::Size;
                        data.clear();
                        break;
                    default:
                        // This should not happen
                        return false;
                }
            } else if (size == UINT32_MAX) {
                switch (current_member) {
                    case Member::Password:
                        password = std::nullopt;
                        current_member = Member::None;
                        current_field = Field::Size;
                        break;
                    default:
                        // continue normally
                        break;
                }
            }
        }
    }

    if (current_member != Member::None) {
        return false;
    }

    result = {url, password};
    return true;
}