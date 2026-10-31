#ifndef MLAPP_FFI_H
#define MLAPP_FFI_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct AppPackageHandle AppPackageHandle;
typedef struct AppSessionHandle AppSessionHandle;

int32_t mlapp_package_open_bytes(const uint8_t* data, size_t len, AppPackageHandle** out_ptr);
void mlapp_package_free(AppPackageHandle* handle);
int32_t mlapp_package_get_manifest_json(AppPackageHandle* handle, char* out_buf, size_t max_len);
int32_t mlapp_package_check_compat_json(AppPackageHandle* handle, char* out_buf, size_t max_len);

int32_t mlapp_session_launch(AppPackageHandle* handle, AppSessionHandle** out_session);
int32_t mlapp_session_send_button_click(AppSessionHandle* session_ptr, const char* comp_name);
void mlapp_session_shutdown(AppSessionHandle* session_ptr);

#ifdef __cplusplus
}
#endif

#endif // MLAPP_FFI_H
