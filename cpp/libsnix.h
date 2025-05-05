#pragma once
#ifdef __cplusplus
extern "C" {
#endif

#include <stdbool.h>
#include <stddef.h>

char *eval_nix_expr(const char *jsonexpr, const char *evalexpr);
void free_cstring(char *s);

#ifdef __cplusplus
}
#endif
