#include "server.h"

int main(void)
{
    int result = 0;
    uint32_t ipv4       = INADDR_ANY;
    unsigned int port   = htons(PORT);

    struct sockaddr_in sock = socket_init(&port, &ipv4);
    int file_descriptor = socket(PF_INET, SOCK_STREAM, 0);

    bind_c(&sock, file_descriptor);
    listen_c(&sock, file_descriptor);

    char *buffer = malloc(SIZE_OF_MESSAGE_BUFFER);



    for(;;){
        struct sockaddr_in client = {0};
        socklen_t client_len = sizeof(client);

        int client_fd = accept_c(&client, &client_len, file_descriptor);

        printf("Client connected! fd: %d\n", client_fd);

        size_t bytes_received = recv(client_fd, buffer, SIZE_OF_MESSAGE_BUFFER, 0);

        buffer[bytes_received] = '\0';

        printf("Message: %s", buffer);

    }

    free(buffer);
}