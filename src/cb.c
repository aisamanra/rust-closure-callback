void call_callback(void (*callback)(int, void*), void* user_data) {
  callback(5, user_data);
}
