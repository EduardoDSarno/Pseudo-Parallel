#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <sys/socket.h>
#include <netinet/in.h>
#include <unistd.h>
#include <ifaddrs.h>
#include <net/if.h>

#define  MAX_TRIES_TO_LISTEN 20

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

int main(void)
{
    int result = 0;
    uint32_t ipv4       = INADDR_ANY;
    unsigned int port   = htons(3490);

    struct sockaddr_in sock = socket_init(&port, &ipv4);
    int file_descriptor = socket(PF_INET, SOCK_STREAM, 0);

    bind_c(&sock, file_descriptor);
    listen_c(&sock, file_descriptor);

    char *buffer = malloc(5000);



    for(;;){
        struct sockaddr_in client = {0};
        socklen_t client_len = sizeof(client);

        int client_fd = accept_c(&client, &client_len, file_descriptor);

        printf("Client connected! fd: %d\n", client_fd);

        size_t bytes_received = recv(client_fd, buffer, 5000, 0);

        buffer[bytes_received] = '\0';

        printf("Message: %s", buffer);

    }

    free(buffer);
}