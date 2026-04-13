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

/*Validates Message Received by recv*/
MessageStatus check_message(char *message, ssize_t bytes_received, int client_fd ){

    if (bytes_received > 0) 
    {
        parse_message(message, bytes_received);
        return MESSAGE_CONTINUE;
    }
    else if (bytes_received == 0) 
    {
        printf("Client closed connection");
        return MESSAGE_CLOSE;
    }

    perror("Error on establishin connection");
    (void)client_fd;
    return MESSAGE_ERROR;
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

    ssize_t bytes_received = recv(client_fd, buffer, SIZE_OF_MESSAGE_BUFFER, 0);
    MessageStatus status = check_message(buffer, bytes_received, client_fd);
    
    if (status != MESSAGE_CONTINUE) {
        free(buffer);
        close(client_fd);
        return NULL;
    }

    int n = 0;

    /*Will loop through the bytes received until end of message token*/
    while (bytes_received < SIZE_OF_MESSAGE_BUFFER) 
    {
        n = recv(client_fd, buffer + bytes_received, 
            (SIZE_OF_MESSAGE_BUFFER - bytes_received),0);
        status = check_message(buffer + bytes_received, n, client_fd);

        if (status != MESSAGE_CONTINUE) 
        {
            break;
        }

        bytes_received += n;
        
        // if found
        if(check_EOL(buffer, bytes_received) == 1) break;
    }

    (void)bytes_received;

    free(buffer);
    close(client_fd);
    return NULL;
}

/* Return 1 if match
    0 if does not*/
int check_EOL(const char *buffer , size_t total_lenght){

    if (total_lenght < 4) {
        return 0;
    }

    for (size_t i = 0; i <= total_lenght - 4; ++i) {
        if( buffer[i]   == '\r' &&
            buffer[i+1] == '\n' &&
            buffer[i+2] == '\r' &&
            buffer[i+3] == '\n') return 1;
    }
        
    
    
    return 0;
    
}