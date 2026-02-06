/*
 * anu_qrn_bits.c
 *
 * Holt quantum random bytes von https://api.quantumnumbers.anu.edu.au/
 * (type=uint8), in Chunks à max. 1024 Bytes/Request, und schreibt die
 * Bitdarstellung (0/1) OHNE Zeilenumbrüche in eine Datei.
 *
 * Build (macOS / Linux):
 *   gcc qrng.c cJSON.c -o anu_qrn_bits -lcurl
 *
 * Usage:
 *   ./anu_qrn_bits -k "APIKEY" -n 4096 -o bits.bin [--raw raw.bin]
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <curl/curl.h>
#include <cjson/cJSON.h>

#define DEFAULT_URL "https://api.quantumnumbers.anu.edu.au/"
#define MAX_PER_REQUEST 1024

typedef struct {
    char *data;
    size_t size;
} MemBuf;

static size_t write_cb(void *contents, size_t size, size_t nmemb, void *userp) {
    size_t realsize = size * nmemb;
    MemBuf *mem = (MemBuf *)userp;

    char *ptr = realloc(mem->data, mem->size + realsize + 1);
    if (!ptr) return 0;

    mem->data = ptr;
    memcpy(mem->data + mem->size, contents, realsize);
    mem->size += realsize;
    mem->data[mem->size] = '\0';
    return realsize;
}

static void byte_to_bits(uint8_t b, char out[8]) {
    for (int i = 7; i >= 0; --i) {
        out[7 - i] = ((b >> i) & 1) ? '1' : '0';
    }
}

static int fetch_chunk_uint8(const char *url, const char *api_key, int length, uint8_t *out) {
    CURL *curl = NULL;
    struct curl_slist *headers = NULL;
    MemBuf mem = {0};

    curl = curl_easy_init();
    if (!curl) return -1;

    char full_url[2048];
    snprintf(full_url, sizeof(full_url), "%s?length=%d&type=uint8", url, length);

    char header_key[512];
    snprintf(header_key, sizeof(header_key), "x-api-key: %s", api_key);
    headers = curl_slist_append(headers, header_key);

    curl_easy_setopt(curl, CURLOPT_URL, full_url);
    curl_easy_setopt(curl, CURLOPT_HTTPHEADER, headers);
    curl_easy_setopt(curl, CURLOPT_WRITEFUNCTION, write_cb);
    curl_easy_setopt(curl, CURLOPT_WRITEDATA, &mem);
    curl_easy_setopt(curl, CURLOPT_USERAGENT, "anu_qrn_bits/1.0");
    curl_easy_setopt(curl, CURLOPT_TIMEOUT, 30L);

    CURLcode res = curl_easy_perform(curl);
    if (res != CURLE_OK) goto cleanup;

    long http_code = 0;
    curl_easy_getinfo(curl, CURLINFO_RESPONSE_CODE, &http_code);
    if (http_code != 200) goto cleanup;

    cJSON *root = cJSON_Parse(mem.data);
    if (!root) goto cleanup;

    cJSON *success = cJSON_GetObjectItemCaseSensitive(root, "success");
    if (!cJSON_IsTrue(success)) {
        cJSON_Delete(root);
        goto cleanup;
    }

    cJSON *data = cJSON_GetObjectItemCaseSensitive(root, "data");
    if (!cJSON_IsArray(data)) {
        cJSON_Delete(root);
        goto cleanup;
    }

    int n = cJSON_GetArraySize(data);
    int to_read = (n < length) ? n : length;

    for (int i = 0; i < to_read; i++) {
        cJSON *item = cJSON_GetArrayItem(data, i);
        if (!cJSON_IsNumber(item)) {
            cJSON_Delete(root);
            goto cleanup;
        }
        int v = item->valueint;
        if (v < 0) v = 0;
        if (v > 255) v = 255;
        out[i] = (uint8_t)v;
    }

    cJSON_Delete(root);
    curl_slist_free_all(headers);
    curl_easy_cleanup(curl);
    free(mem.data);
    return to_read;

cleanup:
    curl_slist_free_all(headers);
    curl_easy_cleanup(curl);
    free(mem.data);
    return -1;
}

int main(int argc, char **argv) {
    const char *api_key = NULL;
    const char *bits_path = "bits.bin";
    const char *raw_path = NULL;
    long total_bytes = -1;

    for (int i = 1; i < argc; i++) {
        if (!strcmp(argv[i], "-k") && i + 1 < argc) {
            api_key = argv[++i];
        } else if (!strcmp(argv[i], "-n") && i + 1 < argc) {
            total_bytes = strtol(argv[++i], NULL, 10);
        } else if (!strcmp(argv[i], "-o") && i + 1 < argc) {
            bits_path = argv[++i];
        } else if (!strcmp(argv[i], "--raw") && i + 1 < argc) {
            raw_path = argv[++i];
        }
    }

    if (!api_key || total_bytes <= 0) {
        fprintf(stderr, "Usage: -k APIKEY -n BYTES [-o bits.bin] [--raw raw.bin]\n");
        return 2;
    }

    curl_global_init(CURL_GLOBAL_DEFAULT);

    FILE *fbits = fopen(bits_path, "wb");
    if (!fbits) return 1;

    FILE *fraw = NULL;
    if (raw_path) fraw = fopen(raw_path, "wb");

    uint8_t *chunk = malloc(MAX_PER_REQUEST);
    if (!chunk) return 1;

    long remaining = total_bytes;
    while (remaining > 0) {
        int want = (remaining > MAX_PER_REQUEST) ? MAX_PER_REQUEST : (int)remaining;
        int got = fetch_chunk_uint8(DEFAULT_URL, api_key, want, chunk);
        if (got <= 0) break;

        for (int i = 0; i < got; i++) {
            char bits[8];
            byte_to_bits(chunk[i], bits);
            fwrite(bits, 1, 8, fbits);
        }

        if (fraw) fwrite(chunk, 1, got, fraw);
        remaining -= got;
    }

    free(chunk);
    fclose(fbits);
    if (fraw) fclose(fraw);
    curl_global_cleanup();

    printf("Done. Wrote %ld bytes (%ld bits) to %s\n",
           total_bytes, total_bytes * 8, bits_path);
    return 0;
}

