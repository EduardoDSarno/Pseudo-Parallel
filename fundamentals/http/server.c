#include "server.h"
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>

struct sockaddr_in socket_init(const unsigned int *port, const uint32_t *ipv4_addr)
{
    struct sockaddr_in dest;
    dest.sin_family = AF_INET;     // Set the address family to IPv4
    dest.sin_port = *port; // right endien
    dest.sin_addr.s_addr = *ipv4_addr;

    return dest;
}

void bind_c(struct sockaddr_in *sockt, int file_descriptor){

    size_t size = sizeof(struct sockaddr_in);

    short result = bind(file_descriptor, (struct sockaddr *)sockt, size);

    if (result < 0) {
        perror("Error binding Socket");
    }

}

void listen_c(struct sockaddr_in *sockt, int file_descriptor){



    size_t size = sizeof(struct sockaddr_in);

    short result = listen(file_descriptor, MAX_TRIES_TO_LISTEN);

    if (result < 0) {
        perror("Error listening to Socket");
    }

}

int accept_c(struct sockaddr_in *other,socklen_t *other_lenght, int file_descriptor){

    int result = accept(file_descriptor,(struct sockaddr *)other, other_lenght);

    if (result < 0) {
        perror("Error listening to Socket");
        return -1;
    }

    return result;

}

int process_message(char *message, ssize_t bytes_received, int client_fd ){

    if (bytes_received > 0) 
    {
        parse_message(message, bytes_received);
    }
    else if (bytes_received == 0) 
    {
        printf("Client closed connection");
    }
    else 
    {
        perror("Error on establishin connection");
        close(client_fd);
        return CONNECTION_ERROR;
    }
    return 0;
}

void parse_message(char *buffer, size_t buffer_lenght){


}

void *worker(void* args){

    int client_fd = (int)(intptr_t)args;

    char *buffer = malloc(SIZE_OF_MESSAGE_BUFFER + 1);
    if (buffer == NULL) {
        perror("Error allocating memory");
        close(client_fd);
        return NULL;
    }

    ssize_t bytes_received = bytes_received = recv(client_fd, buffer,           SIZE_OF_MESSAGE_BUFFER,0); 
    int n = 0;
    

    
    while (bytes_received < SIZE_OF_MESSAGE_BUFFER &&
           strstr(buffer, "\r\n\r\n") != NULL) 
    {
        n = recv(client_fd, buffer,SIZE_OF_MESSAGE_BUFFER,0);
        process_message(buffer, bytes_received, client_fd);

        bytes_received += n;
    }

    
    (void)bytes_received;

    free(buffer);
    close(client_fd);
    return NULL;
}

