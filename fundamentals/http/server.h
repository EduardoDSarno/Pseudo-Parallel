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

struct sockaddr_in socket_init(const unsigned int *port, const uint32_t *ipv4_addr);
void bind_c(struct sockaddr_in *sockt, int file_descriptor);
void listen_c(struct sockaddr_in *sockt, int file_descriptor);
int accept_c(struct sockaddr_in *other,socklen_t *other_lenght, int file_descriptor);
void parse_message(char *buffer,  size_t buffer_lenght);
MessageStatus check_message(char *message, ssize_t bytes_received, int client_fd);
void *worker (void*);
int check_EOL(const char *buffer , size_t total_lenght);

typedef struct ThreadArgs
{


} ThreadArgs;
#endif
