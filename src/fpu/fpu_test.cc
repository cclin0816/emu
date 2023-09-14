#include <array>
#include <cfenv>
#include <cmath>
#include <cstdint>
#include <format>
#include <iostream>
#include <string_view>

#define NOINLINE __attribute__((noinline))

using u8 = uint8_t;
using u32 = uint32_t;
using i32 = int32_t;
using f32 = float;
using u64 = uint64_t;
using i64 = int64_t;
using f64 = double;

using namespace std::literals;

union f32_t {
  u32 u;
  f32 f;
};

union f64_t {
  u64 u;
  f64 f;
};

bool is_nan(f32_t fp) {
  bool exp_all_set = (fp.u & 0x7f800000) == 0x7f800000;
  bool frac_not_all_zero = (fp.u & 0x7fffff) != 0;
  return exp_all_set && frac_not_all_zero;
}
bool is_quiet(f32_t fp) { return fp.u & 0x400000; }
bool is_neg(f32_t fp) { return fp.u & 0x80000000; }
bool is_nan(f64_t fp) {
  bool exp_all_set = (fp.u & 0x7ff0000000000000) == 0x7ff0000000000000;
  bool frac_not_all_zero = (fp.u & 0xfffffffffffff) != 0;
  return exp_all_set && frac_not_all_zero;
}
bool is_quiet(f64_t fp) { return fp.u & 0x8000000000000; }
bool is_neg(f64_t fp) { return fp.u & 0x8000000000000000; }

NOINLINE void print_fp(auto fp) {
  if (is_nan(fp)) {
    if (is_quiet(fp)) {
      if (is_neg(fp)) {
        std::cout << " -qNan  ";
      } else {
        std::cout << "  qNan  ";
      }
    } else {
      if (is_neg(fp)) {
        std::cout << " -sNan  ";
      } else {
        std::cout << "  sNan  ";
      }
    }
  } else {
    std::cout << std::format("{:^8.0e}", fp.f);
  }
}

NOINLINE void set_color(int ec) {
  switch (ec) {
    case 0:
      break;
    case FE_INVALID:
      std::cout << "\033[1;31m";
      break;
    case FE_DIVBYZERO:
      std::cout << "\033[1;35m";
      break;
    case FE_INEXACT:
      std::cout << "\033[1;32m";
      break;
    case FE_OVERFLOW | FE_INEXACT:
      std::cout << "\033[1;34m";
      break;
    case FE_UNDERFLOW | FE_INEXACT:
      std::cout << "\033[1;33m";
      break;
    default:
      std::cerr << std::format("[[EC: {:x}]]", ec);
      break;
  }
}
NOINLINE void reset_color() { std::cout << "\033[0m"; }
NOINLINE int get_ec() { return std::fetestexcept(FE_ALL_EXCEPT); }
NOINLINE int clear_ec() { return std::feclearexcept(FE_ALL_EXCEPT); }

template <typename T>
NOINLINE T cast(auto a) {
  return T(a);
}

template <typename T>
NOINLINE T bitcast(auto a) {
  return std::bit_cast<T>(a);
}

template <typename T>
NOINLINE void print_cast(auto a) {
  clear_ec();
  auto res = cast<T>(a);
  int ec = get_ec();
  set_color(ec);
  std::cout << std::format("{}", res);
  reset_color();
}

template <typename T>
NOINLINE void print_bitcast(auto a) {
  clear_ec();
  auto res = bitcast<T>(a);
  int ec = get_ec();
  set_color(ec);
  std::cout << std::format("{}", res);
  reset_color();
}

int main() {
  std::array<std::string_view, 14> fp_name{
      "-qNan", "-sNan", "-Inf", "-big", "-1",  "-tiny", "-0",
      "0",     "tiny",  "1",    "big",  "Inf", "sNan",  "qNan"};
  std::array<f32_t, 14> fp_arr;
  fp_arr[0].u = 0xffc00000;
  fp_arr[1].u = 0xff800001;
  fp_arr[2].u = 0xff800000;
  fp_arr[3].u = 0xff7fffff;
  fp_arr[4].u = 0xbf800000;
  fp_arr[5].u = 0x80000001;
  fp_arr[6].u = 0x80000000;
  fp_arr[7].u = 0x00000000;
  fp_arr[8].u = 0x00000001;
  fp_arr[9].u = 0x3f800000;
  fp_arr[10].u = 0x7f7fffff;
  fp_arr[11].u = 0x7f800000;
  fp_arr[12].u = 0x7f800001;
  fp_arr[13].u = 0x7fc00000;
  
  print_bitcast<f32>(fp_arr[12].u);
}