#include "server.h"

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
