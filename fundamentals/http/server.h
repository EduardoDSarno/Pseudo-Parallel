#ifndef SERVER_H
#define SERVER_H
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <sys/socket.h>
#include <netinet/in.h>
#include <unistd.h>
#include <pthread.h>
#include <ifaddrs.h>
#include <net/if.h>
#include "header.h"

#define  MAX_TRIES_TO_LISTEN 20
#define  SIZE_OF_MESSAGE_BUFFER 5000
#define  PORT 3490

typedef enum MessageStatus {
    MESSAGE_CONTINUE = 0,
    MESSAGE_CLOSE = 1,
    MESSAGE_ERROR = -1,
} MessageStatus;

struct HttpRequest;

struct sockaddr_in socket_init(const unsigned int *port, const uint32_t *ipv4_addr);
void bind_c(struct sockaddr_in *sockt, int file_descriptor);
void listen_c(struct sockaddr_in *sockt, int file_descriptor);
int accept_c(struct sockaddr_in *other,socklen_t *other_lenght, int file_descriptor);
void parse_message(struct HttpRequest * ,char *buffer,  size_t buffer_lenght);
int get_request_line(struct HttpRequest *http_req, char *buffer, size_t buffer_lenght);
MessageStatus check_message(char *message, ssize_t bytes_received, int client_fd);
void *worker (void*);
int check_EOL(const char *buffer , size_t total_lenght);

enum HttpMethodTyp {
    HTTP_UNKNOWN = 0,
    HTTP_GET,
    HTTP_POST,
    HTTP_PUT,
    HTTP_PATCH,
    HTTP_DELETE,
  };
  
  struct HttpMethod {
    const char *const str;
    enum HttpMethodTyp typ;
  };
  
  static const char HTTP_VERSION[] = "HTTP/1.1";
  
  static const char *const STR_HTTP_GET = "GET";
  static const char *const STR_HTTP_POST = "POST";
  static const char *const STR_HTTP_PUT = "PUT";
  static const char *const STR_HTTP_PATCH = "PATCH";
  static const char *const STR_HTTP_DELETE = "DELETE";
  
  // Table converter string ENUM
  static struct HttpMethod KNOWN_HTTP_METHODS[] = {
      {.str = STR_HTTP_GET, .typ = HTTP_GET},
      {.str = STR_HTTP_POST, .typ = HTTP_POST},
      {.str = STR_HTTP_PUT, .typ = HTTP_PUT},
      {.str = STR_HTTP_PATCH, .typ = HTTP_PATCH},
      {.str = STR_HTTP_DELETE, .typ = HTTP_DELETE},
  };
  
  static const size_t KNOWN_HTTP_METHODS_LEN =
      sizeof(KNOWN_HTTP_METHODS) / sizeof(KNOWN_HTTP_METHODS[0]);
  
  #define BUFFER_SIZE 1024
  #define MAX_HEADERS 128
  
  struct HttpHeader {
    char *key;
    char *value;
  };
  
  struct HttpRequest {
    enum HttpMethodTyp method;
    char *path;
  
    // headers (name, value)
    struct HttpHeader *headers;
    size_t headers_len;
  
    // the request buffer
    char *_buffer;
    size_t _buffer_len;
  
    struct Bstring *body;
  };

#endif
// POST /orders HTTP/1.1\r\n
// Host: localhost:3490\r\n
// Content-Type: application/json\r\n
// Content-Length: 18\r\n
// \r\n
// {"pair":"ETH/USDC"}