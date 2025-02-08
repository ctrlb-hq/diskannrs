#include <immintrin.h>
#include <math.h>
#include <stdint.h>

inline __m256i load_128bit_to_256bit(const __m128i *ptr) {
    __m128i value128 = _mm_loadu_si128(ptr);
    __m256i value256 = _mm256_castsi128_si256(value128);
    return _mm256_inserti128_si256(value256, _mm_setzero_si128(), 1);
}

float distance_compare_avx2_f16(const unsigned char *vec1, const unsigned char *vec2, size_t size) {
    __m256 sum_squared_diff = _mm256_setzero_ps();

    size_t i = 0;
    for (; i <= size - 8; i += 8) {
        __m128i v1_packed = _mm_loadu_si128((__m128i *)(vec1 + i * 2));
        __m128i v2_packed = _mm_loadu_si128((__m128i *)(vec2 + i * 2));

        __m256 v1 = _mm256_cvtph_ps(v1_packed);
        __m256 v2 = _mm256_cvtph_ps(v2_packed);

        __m256 diff = _mm256_sub_ps(v1, v2);
        sum_squared_diff = _mm256_add_ps(sum_squared_diff, _mm256_mul_ps(diff, diff));
    }

    for (; i < size; i++) {
        uint16_t v1 = (vec1[i] << 8) | vec1[i + 1];
        uint16_t v2 = (vec2[i] << 8) | vec2[i + 1];

        union {
            uint16_t in;
            float out;
        } conv1, conv2;

        conv1.in = v1;
        conv2.in = v2;

        float fv1 = conv1.out;
        float fv2 = conv2.out;

        float diff = fv1 - fv2;
        sum_squared_diff = _mm256_add_ps(sum_squared_diff, _mm256_set1_ps(diff * diff));
        i++;
    }

    __m128 sum_low = _mm256_castps256_ps128(sum_squared_diff);
    __m128 sum_high = _mm256_extractf128_ps(sum_squared_diff, 1);
    sum_low = _mm_add_ps(sum_low, sum_high);

    float result[4];
    _mm_storeu_ps(result, sum_low);
    return result[0] + result[1] + result[2] + result[3];
}